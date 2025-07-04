mod api;
mod dto;

use crate::api::{APIClient, EditMember, EditNetwork};
use crate::dto::{
    MemberResponse, NetworkIPAssignmentPool, NetworkResponse, NetworkRoute, NetworkV4AssignMode,
    StatusResponse,
};
use chrono::DateTime;
use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use futures::future::try_join_all;
use std::process::exit;
use std::{env, io};

struct State {
    client: APIClient,
    networks: Vec<NetworkResponse>,
    members: Option<Vec<MemberResponse>>,
    selected_network: Option<usize>,
    status: StatusResponse,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = APIClient::new(
        &*env::var("TOKEN").unwrap_or_default(),
        &*env::var("URL").unwrap_or("http://localhost:9993".to_string()),
    )
    .unwrap();

    println!("{}", "⏳ Fetching networks".yellow());

    let status = match client.status().await {
        Ok(r) => r,
        Err(e) => {
            println!("❌ Request failed: {}", e);
            exit(0);
        }
    };

    println!("Node ID: {}", status.address);
    println!(
        "{}",
        "Use arrows to navigate up & down. Use `q` to return back.".bright_magenta()
    );

    let network_ids = client.networks().await.unwrap();
    let network_futures = network_ids.iter().map(|id| client.network(id));
    let networks = try_join_all(network_futures).await.unwrap();

    let mut state = State {
        status,
        client,
        networks,
        selected_network: None,
        members: None,
    };

    loop {
        if state.selected_network.is_some() {
            if state.members.is_some() {
                members_list(&mut state).await;
            } else {
                network_options(&mut state).await;
            }
        } else {
            networks_list(&mut state).await;
        }
    }
}

async fn networks_list(state: &mut State) {
    let mut items = state
        .networks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    items.push("Create new network...".to_string());
    items.push("Exit".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Networks")
        .items(&*items)
        .default(0)
        .interact_opt()
        .unwrap();

    match selection {
        Some(index) => {
            if index == state.networks.len() {
                match state.client.create_network(&*state.status.address).await {
                    Ok(r) => {
                        println!("⚡ Network created: {}", r.id);
                        state.networks.push(r)
                    }
                    Err(e) => {
                        println!("❌ Request failed: {}", e)
                    }
                }
                return;
            }

            if index == state.networks.len() + 1 {
                exit(0);
            }

            state.selected_network = Some(index);
        }
        None => {}
    }
}

async fn network_options(state: &mut State) {
    let selected_index = match state.selected_network {
        Some(index) => index,
        None => return,
    };
    let network_id = state.networks[selected_index].id.clone();
    let network_name = state.networks[selected_index].to_string();
    let zt_mode = state.networks[selected_index].v4_assign_mode.zt;

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Network {}", network_name))
        .items(&[
            "Info",
            "Members",
            "Rename",
            "Set ip assignment pool",
            "Set route",
            if zt_mode {
                "Disable v4 ZT Mode"
            } else {
                "Enable v4 ZT Mode"
            },
            "Delete",
        ])
        .default(0)
        .interact_opt()
        .unwrap();

    match selection {
        Some(index) => match index {
            0 => {
                let network = &state.networks[selected_index];

                println!(
                    "\nID: {}\nName: {}\nCreation Date: {}\nIs Private: {}\nRoutes: {}\nIP Assignment Pools: {}\nIs ZT V4 Assign Mode: {}",
                    network.id,
                    network.name.as_ref().unwrap_or(&"Not set".to_string()),
                    DateTime::from_timestamp_millis(network.creation_time)
                        .unwrap()
                        .to_rfc3339(),
                    network.private,
                    network
                        .routes
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                    network
                        .ip_assignment_pools
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                    network.v4_assign_mode.zt
                )
            }
            1 => {
                println!("{}", "⏳ Fetching members".yellow());
                let member_ids = state.client.members(&*network_id).await.unwrap();
                if member_ids.is_empty() {
                    println!(
                        "{}",
                        "🌧  This network doesn't contains any members".bright_blue()
                    );
                    return;
                }
                let member_futures = member_ids
                    .iter()
                    .map(|id| state.client.member(&*network_id, id.0));
                let members = try_join_all(member_futures).await.unwrap();

                state.members = Some(members);
            }
            2 => {
                let network = &mut state.networks[selected_index];
                let name: String = Input::new()
                    .with_prompt("New name")
                    .interact_text()
                    .unwrap_or("".to_string());
                if let Err(e) = state
                    .client
                    .edit_network(
                        &*network_id,
                        EditNetwork {
                            name: Some(name.clone()),
                            ip_assignment_pools: network.ip_assignment_pools.clone(),
                            private: network.private,
                            routes: network.routes.clone(),
                            v4_assign_mode: network.v4_assign_mode.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    network.name = Some(name);
                    println!("{}", "✔ Network updated".bright_green());
                }
            }
            3 => {
                let network = &mut state.networks[selected_index];
                let start: String = Input::new()
                    .with_prompt("IP Range Start")
                    .default("192.168.192.1".to_string())
                    .interact_text()
                    .unwrap_or("".to_string());
                let end: String = Input::new()
                    .with_prompt("IP Range Start")
                    .default("192.168.192.254".to_string())
                    .interact_text()
                    .unwrap_or("".to_string());

                let ips = vec![NetworkIPAssignmentPool {
                    ip_range_start: start,
                    ip_range_end: end,
                }];

                if let Err(e) = state
                    .client
                    .edit_network(
                        &*network_id,
                        EditNetwork {
                            name: network.name.clone(),
                            ip_assignment_pools: ips.clone(),
                            private: network.private,
                            routes: network.routes.clone(),
                            v4_assign_mode: network.v4_assign_mode.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    network.ip_assignment_pools = ips;
                    println!("{}", "✔ Network updated".bright_green());
                }
            }
            4 => {
                let network = &mut state.networks[selected_index];
                let target: String = Input::new()
                    .with_prompt("Target")
                    .default("192.168.192.0/24".to_string())
                    .interact_text()
                    .unwrap_or("".to_string());
                let via: String = Input::new()
                    .with_prompt("Via (empty is null)")
                    .default("".to_string())
                    .interact_text()
                    .unwrap_or("".to_string());

                let routes = vec![NetworkRoute {
                    target,
                    via: if via.is_empty() { None } else { Some(via) },
                }];

                if let Err(e) = state
                    .client
                    .edit_network(
                        &*network_id,
                        EditNetwork {
                            name: network.name.clone(),
                            ip_assignment_pools: network.ip_assignment_pools.clone(),
                            private: network.private,
                            routes: routes.clone(),
                            v4_assign_mode: network.v4_assign_mode.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    network.routes = routes;
                    println!("{}", "✔ Network updated".bright_green());
                }
            }
            5 => {
                let network = &mut state.networks[selected_index];
                if let Err(e) = state
                    .client
                    .edit_network(
                        &*network_id,
                        EditNetwork {
                            name: network.name.clone(),
                            ip_assignment_pools: network.ip_assignment_pools.clone(),
                            private: network.private,
                            routes: network.routes.clone(),
                            v4_assign_mode: NetworkV4AssignMode {
                                zt: !network.v4_assign_mode.zt,
                            },
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    network.v4_assign_mode = NetworkV4AssignMode {
                        zt: !network.v4_assign_mode.zt,
                    };
                    println!("{}", "✔ Network updated".bright_green());
                }
            }
            6 => {
                if Confirm::new()
                    .with_prompt("Are you sure want to delete this network?")
                    .interact()
                    .unwrap()
                {
                    if let Err(e) = state.client.delete_network(&*network_id).await {
                        println!("❌ Request failed: {}", e)
                    }

                    println!("{}", "✔ Network deleted".bright_green());
                    state.selected_network = None;
                    state.networks.remove(
                        state
                            .networks
                            .iter()
                            .position(|x| x.id == network_id)
                            .unwrap(),
                    );
                }
            }
            _ => {}
        },
        None => state.selected_network = None,
    }
}

async fn members_list(state: &mut State) {
    let index = {
        let members = state.members.as_ref().unwrap();
        let items = members
            .iter()
            .map(|x| {
                let s = format!(
                    "{} {}",
                    if !x.authorized { "🔒" } else { "🔓" },
                    x.to_string()
                );
                if x.authorized {
                    s.bright_green()
                } else {
                    s.bright_red()
                }
            })
            .collect::<Vec<ColoredString>>();

        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Members of {}",
                state.networks[state.selected_network.unwrap()].to_string()
            ))
            .items(&*items)
            .default(0)
            .interact_opt()
            .unwrap()
    };

    match index {
        Some(index) => member_options(state, index).await,
        None => state.members = None,
    }
}

async fn member_options(state: &mut State, index: usize) {
    let members = state.members.as_mut().unwrap();
    let member = &mut members[index];
    let network = &state.networks[state.selected_network.unwrap()];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Member {} of {}",
            member.to_string(),
            network.to_string()
        ))
        .items(&[
            if member.authorized {
                "Deauthorize"
            } else {
                "Authorize"
            },
            "Set name",
            "Set IP",
            "Delete",
        ])
        .default(0)
        .interact_opt()
        .unwrap();

    match selection {
        Some(i) => match i {
            0 => {
                if let Err(e) = state
                    .client
                    .edit_member(
                        &*member.nwid,
                        &*member.id,
                        EditMember {
                            authorized: !member.authorized,
                            ip_assignments: member.ip_assignments.clone(),
                            name: member.name.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    member.authorized = !member.authorized;
                    println!("{}", "✔ Member updated".bright_green());
                }
            }
            1 => {
                let name: String = match Input::new().with_prompt("New name").interact_text() {
                    Ok(e) => e,
                    Err(e) => {
                        return;
                    }
                };

                let name = if name.is_empty() { None } else { Some(name) };

                if let Err(e) = state
                    .client
                    .edit_member(
                        &*member.nwid,
                        &*member.id,
                        EditMember {
                            authorized: member.authorized,
                            ip_assignments: member.ip_assignments.clone(),
                            name: name.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    member.name = name;
                    println!("{}", "✔ Member updated".bright_green());
                }
            }
            2 => {
                let ip: String = match Input::new().with_prompt("New IP").interact_text() {
                    Ok(e) => e,
                    Err(e) => {
                        return;
                    }
                };

                let mut ips = member.ip_assignments.clone();
                ips.clear();
                ips.push(ip);

                if let Err(e) = state
                    .client
                    .edit_member(
                        &*member.nwid,
                        &*member.id,
                        EditMember {
                            authorized: member.authorized,
                            ip_assignments: ips.clone(),
                            name: member.name.clone(),
                        },
                    )
                    .await
                {
                    println!("❌ Request failed: {}", e)
                } else {
                    member.ip_assignments = ips;
                    println!("{}", "✔ Member updated".bright_green());
                }
            }
            3 => {
                if let Err(e) = state.client.delete_member(&*member.nwid, &*member.id).await {
                    println!("❌ Request failed: {}", e)
                } else {
                    members.remove(index);
                    println!("{}", "✔ Member deleted".bright_green());
                }
            }
            _ => {}
        },
        None => {}
    }
}
