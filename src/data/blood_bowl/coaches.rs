use crate::AppState;
use crate::data::blood_bowl::competitions::Competition;
use crate::data::users::User;
use crate::errors::AppError;
use blood_bowl_rs::coaches::Coach;
use blood_bowl_rs::elo;
use blood_bowl_rs::elo::NAF_INITIAL_ELO;
use std::collections::HashMap;

pub async fn select_by_id(state: &AppState, id: Option<i32>) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_by_id with id={:?}", id);

    let coach = User::select_by_id(state, id).await?;
    Ok(coach.and_then(|user| Some(user.into())))
}

pub async fn select_from_team(state: &AppState, team_id: i32) -> Result<Option<Coach>, AppError> {
    tracing::debug!("select_from_team with team_id={}", team_id);

    let coach: Option<User> = sqlx::query_as(
        "SELECT users.id,
                users.email,
                users.name,
                users.given_name,
                users.family_name,
                users.picture
        FROM users
        INNER JOIN bb_teams
        ON users.id = bb_teams.coach_id
        WHERE bb_teams.id = $1
        LIMIT 1",
    )
    .bind(team_id.clone())
    .fetch_optional(&state.db)
    .await?;

    Ok(coach.and_then(|user| Some(user.into())))
}

pub async fn select_elo_ranking(state: &AppState) -> Result<Vec<Coach>, AppError> {
    tracing::debug!("compute_elo_ranking with state");

    let mut games = crate::data::blood_bowl::games::select_all_games_played(state).await?;

    let mut coaches: HashMap<i32, Coach> = HashMap::new();

    for game in games.iter_mut() {
        let competition_team_number =
            if let Some(competition) = Competition::select_for_game_id(state, game.id).await? {
                Some(competition.select_playing_teams(state).await?.len())
            } else {
                None
            };

        let first_coach_in_list = game.first_team.coach.id.map(|id| {
            coaches
                .entry(id)
                .or_insert_with(|| game.first_team.coach.clone())
        });

        if let Some(coach_in_list) = first_coach_in_list {
            game.first_team.coach = coach_in_list.clone();
        }

        let second_coach_in_list = game.second_team.coach.id.map(|id| {
            coaches
                .entry(id)
                .or_insert_with(|| game.second_team.coach.clone())
        });

        if let Some(coach_in_list) = second_coach_in_list {
            game.second_team.coach = coach_in_list.clone();
        }

        let (first_coach_new_elo, second_coach_new_elo) =
            elo::new_naf_elo_from_game(game, competition_team_number, competition_team_number);

        if let Some(coach_id) = game.first_team.coach.id {
            coaches
                .entry(coach_id)
                .and_modify(|coach| coach.elo = Some(first_coach_new_elo));
        }

        if let Some(coach_id) = game.second_team.coach.id {
            coaches
                .entry(coach_id)
                .and_modify(|coach| coach.elo = Some(second_coach_new_elo));
        }
    }

    let mut ranking: Vec<Coach> = coaches.into_values().collect();

    ranking.sort_by(|a, b| {
        b.elo
            .unwrap_or(NAF_INITIAL_ELO)
            .partial_cmp(&a.elo.unwrap_or(NAF_INITIAL_ELO))
            .unwrap()
    });

    Ok(ranking)
}
