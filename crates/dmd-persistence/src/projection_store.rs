use dmd_domain::{CampaignId, CampaignState};
use sqlx::{Row, SqlitePool};
use thiserror::Error;

use crate::{ReplayEventApplier, SnapshotReplayError, replay_campaign_to_head};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignProjectionSummary {
    pub campaign_id: CampaignId,
    pub state_schema_version: u32,
    pub applied_event_sequence: u64,
    pub players: u64,
    pub characters: u64,
    pub entities: u64,
    pub factions: u64,
    pub locations: u64,
    pub scenes: u64,
    pub scene_presences: u64,
    pub items: u64,
    pub facts: u64,
    pub claims: u64,
    pub beliefs: u64,
    pub knowledge: u64,
    pub directives: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedEntity {
    pub entity_id: String,
    pub display_name: String,
    pub existence: String,
    pub location_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum ProjectionStoreError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Replay(#[from] SnapshotReplayError),
    #[error("campaign state is not initialized")]
    CampaignNotInitialized,
    #[error("projection state is missing for initialized campaign")]
    MissingProjection,
    #[error(
        "projection head does not match authoritative materialized state: projection={projection_sequence}@{projection_schema}, authoritative={authoritative_sequence}@{authoritative_schema}"
    )]
    HeadMismatch {
        projection_sequence: u64,
        projection_schema: u32,
        authoritative_sequence: u64,
        authoritative_schema: u32,
    },
    #[error("projection row counts do not match their materialized head")]
    CountMismatch,
    #[error("invalid stored projection integer in {field}: {value}")]
    InvalidInteger { field: &'static str, value: i64 },
    #[error("replayed campaign state could not be encoded for projection rebuild: {0}")]
    StateSerialization(String),
    #[error("campaign head changed while projection rebuild was being prepared")]
    RebuildHeadChanged,
}

/// Loads the campaign's normalized projection head and mechanically verifies that it mirrors the
/// authoritative materialized head and that every projected record-family count still matches.
pub async fn load_campaign_projection_summary(
    pool: &SqlitePool,
    campaign_id: CampaignId,
) -> Result<CampaignProjectionSummary, ProjectionStoreError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT p.state_schema_version, p.applied_event_sequence,
               p.players_count, p.characters_count, p.entities_count, p.factions_count,
               p.locations_count, p.scenes_count, p.scene_presences_count, p.items_count,
               p.facts_count, p.claims_count, p.beliefs_count, p.knowledge_count,
               p.directives_count,
               c.schema_version AS authoritative_schema,
               c.applied_event_sequence AS authoritative_sequence
        FROM projection_heads p
        JOIN campaign_state_current c ON c.campaign_id = p.campaign_id
        WHERE p.campaign_id = ?
        "#,
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(&mut *transaction)
    .await?;

    let Some(row) = row else {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM campaign_state_current WHERE campaign_id = ?",
        )
        .bind(campaign_id.0.to_string())
        .fetch_optional(&mut *transaction)
        .await?;
        transaction.rollback().await?;
        return if exists.is_some() {
            Err(ProjectionStoreError::MissingProjection)
        } else {
            Err(ProjectionStoreError::CampaignNotInitialized)
        };
    };

    let projection_schema = stored_u32(
        row.try_get("state_schema_version")?,
        "projection_heads.state_schema_version",
    )?;
    let projection_sequence = stored_u64(
        row.try_get("applied_event_sequence")?,
        "projection_heads.applied_event_sequence",
    )?;
    let authoritative_schema = stored_u32(
        row.try_get("authoritative_schema")?,
        "campaign_state_current.schema_version",
    )?;
    let authoritative_sequence = stored_u64(
        row.try_get("authoritative_sequence")?,
        "campaign_state_current.applied_event_sequence",
    )?;
    if projection_schema != authoritative_schema || projection_sequence != authoritative_sequence {
        transaction.rollback().await?;
        return Err(ProjectionStoreError::HeadMismatch {
            projection_sequence,
            projection_schema,
            authoritative_sequence,
            authoritative_schema,
        });
    }

    let summary = CampaignProjectionSummary {
        campaign_id,
        state_schema_version: projection_schema,
        applied_event_sequence: projection_sequence,
        players: stored_u64(
            row.try_get("players_count")?,
            "projection_heads.players_count",
        )?,
        characters: stored_u64(
            row.try_get("characters_count")?,
            "projection_heads.characters_count",
        )?,
        entities: stored_u64(
            row.try_get("entities_count")?,
            "projection_heads.entities_count",
        )?,
        factions: stored_u64(
            row.try_get("factions_count")?,
            "projection_heads.factions_count",
        )?,
        locations: stored_u64(
            row.try_get("locations_count")?,
            "projection_heads.locations_count",
        )?,
        scenes: stored_u64(
            row.try_get("scenes_count")?,
            "projection_heads.scenes_count",
        )?,
        scene_presences: stored_u64(
            row.try_get("scene_presences_count")?,
            "projection_heads.scene_presences_count",
        )?,
        items: stored_u64(row.try_get("items_count")?, "projection_heads.items_count")?,
        facts: stored_u64(row.try_get("facts_count")?, "projection_heads.facts_count")?,
        claims: stored_u64(
            row.try_get("claims_count")?,
            "projection_heads.claims_count",
        )?,
        beliefs: stored_u64(
            row.try_get("beliefs_count")?,
            "projection_heads.beliefs_count",
        )?,
        knowledge: stored_u64(
            row.try_get("knowledge_count")?,
            "projection_heads.knowledge_count",
        )?,
        directives: stored_u64(
            row.try_get("directives_count")?,
            "projection_heads.directives_count",
        )?,
    };

    let actual = load_actual_counts(&mut transaction, campaign_id).await?;
    if actual != summary_counts(&summary) {
        transaction.rollback().await?;
        return Err(ProjectionStoreError::CountMismatch);
    }
    transaction.commit().await?;
    Ok(summary)
}

/// Example production query surface: locate current world entities without decoding CampaignState.
pub async fn list_projected_entities_at_location(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    location_id: Option<&str>,
) -> Result<Vec<ProjectedEntity>, ProjectionStoreError> {
    load_campaign_projection_summary(pool, campaign_id).await?;
    let rows = if let Some(location_id) = location_id {
        sqlx::query(
            "SELECT entity_id, display_name, existence, location_id FROM projection_entities WHERE campaign_id = ? AND location_id = ? ORDER BY entity_id",
        )
        .bind(campaign_id.0.to_string())
        .bind(location_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT entity_id, display_name, existence, location_id FROM projection_entities WHERE campaign_id = ? AND location_id IS NULL ORDER BY entity_id",
        )
        .bind(campaign_id.0.to_string())
        .fetch_all(pool)
        .await?
    };

    rows.into_iter()
        .map(|row| {
            Ok(ProjectedEntity {
                entity_id: row.try_get("entity_id")?,
                display_name: row.try_get("display_name")?,
                existence: row.try_get("existence")?,
                location_id: row.try_get("location_id")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ProjectionStoreError::from)
}

/// Deterministically repairs projection drift from the accepted snapshot+journal recovery path.
/// Replay remains the authority. The materialized recovery row is replaced only if its sequence is
/// unchanged from the value observed before replay; the SQLite projection trigger then replaces all
/// derivative rows in that same statement transaction. No command/event/snapshot history is edited.
pub async fn rebuild_campaign_projections(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    applier: &dyn ReplayEventApplier,
) -> Result<CampaignProjectionSummary, ProjectionStoreError> {
    let observed_sequence = sqlx::query_scalar::<_, i64>(
        "SELECT applied_event_sequence FROM campaign_state_current WHERE campaign_id = ?",
    )
    .bind(campaign_id.0.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(ProjectionStoreError::CampaignNotInitialized)?;
    let state = replay_campaign_to_head(pool, campaign_id, applier).await?;
    rebuild_from_replayed_state(pool, campaign_id, observed_sequence, &state).await?;
    load_campaign_projection_summary(pool, campaign_id).await
}

async fn rebuild_from_replayed_state(
    pool: &SqlitePool,
    campaign_id: CampaignId,
    observed_sequence: i64,
    state: &CampaignState,
) -> Result<(), ProjectionStoreError> {
    let state_json = state
        .encode_json()
        .map_err(|error| ProjectionStoreError::StateSerialization(error.to_string()))?;
    let applied_sequence = sequence_to_i64(state.applied_event_sequence)?;
    let updated = sqlx::query(
        "UPDATE campaign_state_current SET schema_version = ?, applied_event_sequence = ?, state_json = ? WHERE campaign_id = ? AND applied_event_sequence = ?",
    )
    .bind(i64::from(state.schema_version))
    .bind(applied_sequence)
    .bind(state_json)
    .bind(campaign_id.0.to_string())
    .bind(observed_sequence)
    .execute(pool)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(ProjectionStoreError::RebuildHeadChanged);
    }
    Ok(())
}

async fn load_actual_counts(
    connection: &mut sqlx::SqliteConnection,
    campaign_id: CampaignId,
) -> Result<[u64; 13], ProjectionStoreError> {
    let row = sqlx::query(
        r#"
        SELECT
          (SELECT COUNT(*) FROM projection_players WHERE campaign_id = ?) players,
          (SELECT COUNT(*) FROM projection_characters WHERE campaign_id = ?) characters,
          (SELECT COUNT(*) FROM projection_entities WHERE campaign_id = ?) entities,
          (SELECT COUNT(*) FROM projection_factions WHERE campaign_id = ?) factions,
          (SELECT COUNT(*) FROM projection_locations WHERE campaign_id = ?) locations,
          (SELECT COUNT(*) FROM projection_scenes WHERE campaign_id = ?) scenes,
          (SELECT COUNT(*) FROM projection_scene_presences WHERE campaign_id = ?) scene_presences,
          (SELECT COUNT(*) FROM projection_items WHERE campaign_id = ?) items,
          (SELECT COUNT(*) FROM projection_facts WHERE campaign_id = ?) facts,
          (SELECT COUNT(*) FROM projection_claims WHERE campaign_id = ?) claims,
          (SELECT COUNT(*) FROM projection_beliefs WHERE campaign_id = ?) beliefs,
          (SELECT COUNT(*) FROM projection_knowledge WHERE campaign_id = ?) knowledge,
          (SELECT COUNT(*) FROM projection_directives WHERE campaign_id = ?) directives
        "#,
    )
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .bind(campaign_id.0.to_string())
    .fetch_one(&mut *connection)
    .await?;
    Ok([
        stored_u64(row.try_get("players")?, "projection_players.count")?,
        stored_u64(row.try_get("characters")?, "projection_characters.count")?,
        stored_u64(row.try_get("entities")?, "projection_entities.count")?,
        stored_u64(row.try_get("factions")?, "projection_factions.count")?,
        stored_u64(row.try_get("locations")?, "projection_locations.count")?,
        stored_u64(row.try_get("scenes")?, "projection_scenes.count")?,
        stored_u64(
            row.try_get("scene_presences")?,
            "projection_scene_presences.count",
        )?,
        stored_u64(row.try_get("items")?, "projection_items.count")?,
        stored_u64(row.try_get("facts")?, "projection_facts.count")?,
        stored_u64(row.try_get("claims")?, "projection_claims.count")?,
        stored_u64(row.try_get("beliefs")?, "projection_beliefs.count")?,
        stored_u64(row.try_get("knowledge")?, "projection_knowledge.count")?,
        stored_u64(row.try_get("directives")?, "projection_directives.count")?,
    ])
}

fn summary_counts(summary: &CampaignProjectionSummary) -> [u64; 13] {
    [
        summary.players,
        summary.characters,
        summary.entities,
        summary.factions,
        summary.locations,
        summary.scenes,
        summary.scene_presences,
        summary.items,
        summary.facts,
        summary.claims,
        summary.beliefs,
        summary.knowledge,
        summary.directives,
    ]
}

fn stored_u64(value: i64, field: &'static str) -> Result<u64, ProjectionStoreError> {
    u64::try_from(value).map_err(|_| ProjectionStoreError::InvalidInteger { field, value })
}

fn stored_u32(value: i64, field: &'static str) -> Result<u32, ProjectionStoreError> {
    u32::try_from(value).map_err(|_| ProjectionStoreError::InvalidInteger { field, value })
}

fn sequence_to_i64(value: u64) -> Result<i64, ProjectionStoreError> {
    i64::try_from(value).map_err(|_| ProjectionStoreError::InvalidInteger {
        field: "applied_event_sequence",
        value: i64::MAX,
    })
}
