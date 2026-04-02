use bevy::prelude::*;
use grass::GrassSystems;
use saddle_bevy_e2e::{E2EPlugin, E2ESet, action::Action, init_scenario};

use crate::scenarios;

pub struct GrassLabE2EPlugin;

impl Plugin for GrassLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.configure_sets(Update, E2ESet.before(GrassSystems::Prepare));

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenarios::scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                    info!("[e2e] Scenario '{name}' loaded with --handoff");
                } else {
                    info!("[e2e] Scenario '{name}' loaded");
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[e2e] Unknown scenario '{name}'. Available: {:?}",
                    scenarios::list_scenarios()
                );
            }
        }
    }
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}
