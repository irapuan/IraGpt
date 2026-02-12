use dialoguer::{theme::ColorfulTheme, MultiSelect};
use good_lp::*;
use is_terminal::IsTerminal;
use std::fs;
use std::io::{self, Read};
mod player;
use player::Player;

fn load_players(filename: &str) -> Vec<Player> {
    if let Ok(data) = fs::read_to_string(filename) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

fn save_teams(selections: &[Player], filename: &str) {
    let serialized = serde_json::to_string(selections).unwrap();
    fs::write(filename, serialized).unwrap();
}

fn load_teams(filename: &str) -> Vec<Player> {
    if let Ok(data) = fs::read_to_string(filename) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

fn balance_teams(
    players: &[Player],
    number_of_teams: usize,
    players_per_team: usize,
) -> Vec<Vec<Player>> {
    let mut variables = variables!();
    let team_allocation_variables: Vec<Vec<_>> = players
        .iter()
        .map(|_| {
            (0..number_of_teams)
                .map(|_| variables.add(variable().name("team".to_owned()).binary()))
                .collect()
        })
        .collect();

    // println!("team allocation: {:?}", team_allocation_variables);
    const CRITERIA: usize = 6;
    let max_diff: Vec<_> = (0..CRITERIA)
        .map(|_| variables.add(variable().min(0.0)))
        .collect();

    let mut lp_problem = variables
        .minimise(max_diff.iter().sum::<Expression>())
        .using(highs);

    for (player_idx, _) in players.iter().enumerate() {
        lp_problem = lp_problem.with(constraint!(
            team_allocation_variables[player_idx]
                .iter()
                .sum::<Expression>()
                == 1
        ));
    }

    for team_idx in 0..number_of_teams {
        let team_size_constraint: Expression = (0..players.len())
            .map(|player_idx| &team_allocation_variables[player_idx][team_idx])
            .sum::<Expression>();
        lp_problem = lp_problem.with(constraint!(team_size_constraint == players_per_team as i32));
    }

    for criteria_idx in 1..CRITERIA {
        let team_scores: Vec<Expression> = (0..number_of_teams)
            .map(|team_idx| {
                (0..players.len())
                    .map(|player_idx| {
                        players[player_idx].qualidades()[criteria_idx] as f64
                            * team_allocation_variables[player_idx][team_idx]
                    })
                    .fold(0.0.into(), |acc, expr| acc + expr)
            })
            .collect();

        let avg_score = team_scores.iter().sum::<Expression>() / number_of_teams as f64;

        for team_idx in 0..number_of_teams {
            lp_problem = lp_problem.with(constraint!(
                max_diff[criteria_idx] >= team_scores[team_idx].clone() - avg_score.clone()
            ));
            lp_problem = lp_problem.with(constraint!(
                max_diff[criteria_idx] >= avg_score.clone() - team_scores[team_idx].clone()
            ));
        }

    }

    let solution = lp_problem.solve().expect("Falha ao resolver");

    let mut teams: Vec<Vec<Player>> = vec![vec![]; number_of_teams];
    for (i, player) in players.iter().enumerate() {
        for j in 0..number_of_teams {
            if solution.value(team_allocation_variables[i][j]) > 0.5 {
                teams[j].push(player.clone());
            }
        }
    }

    // println!("Max diff goal: {},", solution.value(max_diff[0]));
    // println!("Max diff zaga: {},", solution.value(max_diff[1]));
    // println!("Max diff meio: {},", solution.value(max_diff[2]));
    // println!("Max diff ataque: {},", solution.value(max_diff[3]));
    // println!("Max diff speed: {},", solution.value(max_diff[4]));
    // println!("Max diff stamina: {},", solution.value(max_diff[5]));
    teams
}

fn main() {
    let players = load_players("players.json");
    let saved_selections_file = "selections.json";
    let saved_selections = load_teams(saved_selections_file);

    // Read list of player names from stdin if available
    let mut stdin_input = String::new();
    if !io::stdin().is_terminal() {
        // Only read if stdin is piped (not interactive terminal)
        io::stdin()
            .read_to_string(&mut stdin_input)
            .expect("Erro ao ler stdin");
    }

    // Normalize and parse stdin list
    let stdin_players: Vec<String> = stdin_input
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    // Determine default selections:
    // Priority: stdin list > previously saved selections
    let defaults: Vec<bool> = players
        .iter()
        .map(|p| {
            if !stdin_players.is_empty() {
                stdin_players.contains(&p.name)
            } else {
                saved_selections.contains(p)
            }
        })
        .collect();
    /* let defaults: Vec<bool> = players
    .iter()
    .map(|j| saved_selections.contains(j))
    .collect(); */

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecione os jogadores para formar os times")
        .items(&players)
        .defaults(&defaults)
        .interact()
        .unwrap();

    let selected_items: Vec<Player> = selections
        .iter()
        .map(|&player_idx| players[player_idx].clone())
        .collect();
    save_teams(&selected_items, saved_selections_file);

    let players_per_team = 5;
    let number_of_teams = selected_items.len() / players_per_team;
    let balanced_teams = balance_teams(&selected_items, number_of_teams, players_per_team);

    let colors = ["Preto", "Azul", "Amarelo", "Laranja"];

    for (i, team) in balanced_teams.iter().enumerate() {
        println!("Time {}:", colors[i % colors.len()]);
        for player in team {
            println!("  {}", player.name);
            // println!(
            //     "{} - Goleiro: {}, Zagueiro: {}, Meio: {}, Atacante: {}, Velocidade: {}, Preparo: {}, Media: {}",
            //     player.name,
            //     player.qualidade_goleiro,
            //     player.qualidade_zagueiro,
            //     player.qualidade_meio,
            //     player.qualidade_atacante,
            //     player.speed,
            //     player.stamina,
            //     player::media_qualidade_jogador(player)
            // );
        }
        // println!("Soma de Qualidade do Time:");
        println!(
            "Media de notas do time: {}",
            player::media_do_jogadores(team)
        );
        println!(
            "  Goleiro: {} - max: {}",
            player::rate_average(team, &player::Criteria::Keeper),
            player::rate_max(team, &player::Criteria::Keeper)
        );
        println!(
            "  Zagueiro: {} - max: {}",
            player::rate_average(team, &player::Criteria::Defender),
            player::rate_max(team, &player::Criteria::Defender)
        );
        println!(
            "  Meio: {} - max: {}",
            player::rate_average(team, &player::Criteria::Midfielder),
            player::rate_max(team, &player::Criteria::Midfielder)
        );
        println!(
            "  Atacante: {} - max: {}",
            player::rate_average(team, &player::Criteria::Forward),
            player::rate_max(team, &player::Criteria::Forward)
        );
        println!(
            "  Velocidade: {} - max: {}",
            player::rate_average(team, &player::Criteria::Speed),
            player::rate_max(team, &player::Criteria::Speed)
        );
        println!(
            "  Stamina: {} - max: {}",
            player::rate_average(team, &player::Criteria::Stamina),
            player::rate_max(team, &player::Criteria::Stamina)
        );
        println!();
    }
    println!(
        "Diferença máxima total entre todos os crtitérios: {}",
        player::total_diference(&balanced_teams)
    );

    println!("--------resultado para copiar e colar-------------");
    for (team_idx, team) in balanced_teams.iter().enumerate() {
        println!("Time {}:", colors[team_idx % colors.len()]);
        for player in team {
            println!("{}", player.name);
        }
        println!();
    }
}
