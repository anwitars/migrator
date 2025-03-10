use std::str::FromStr;

use crate::{Migration, migrations::MigrationId};

// TODO: clean up the naming mess
#[derive(Debug, Clone)]
pub enum Revision {
    Relative(RelativeRevision),
    Absolute(String),
}

#[derive(Debug, Clone)]
pub enum RelativeRevision {
    Head(i32),
    Current(i32),
}

impl Revision {
    fn resolve_revision_index(
        &self,
        all_migrations: &[Migration],
        current: Option<usize>,
    ) -> Result<usize, &'static str> {
        match self {
            Revision::Absolute(revision) => all_migrations
                .iter()
                .position(|m| m.stringify_id() == *revision)
                .ok_or("Invalid revision"),

            Revision::Relative(revision) => match revision {
                RelativeRevision::Head(offset) => {
                    if offset < &0 {
                        return Err("Offset must be positive");
                    }

                    let head_revision = all_migrations.len() as i32 - 1;
                    let target_index = head_revision - offset;

                    if target_index < 0 {
                        return Err("Offset is too large");
                    }

                    Ok(target_index as usize)
                }
                RelativeRevision::Current(offset) => {
                    let current_index = current.ok_or("No current migration")?;
                    let target_index = current_index as i32 + offset;

                    if target_index < 0 {
                        return Err("Offset is too large");
                    }

                    if target_index >= all_migrations.len() as i32 {
                        return Err("Offset is too large");
                    }

                    Ok(target_index as usize)
                }
            },
        }
    }

    pub fn resolve_revision_id(
        &self,
        all_migrations: &[Migration],
    ) -> Result<String, &'static str> {
        let target_index = self.resolve_revision_index(all_migrations, None)?;
        Ok(all_migrations[target_index].stringify_id().clone())
    }

    fn resolve_current_index(
        &self,
        all_migrations: &[Migration],
        current: Option<&MigrationId>,
    ) -> Result<Option<usize>, &'static str> {
        let current_index = match current {
            Some(current) => Some(
                all_migrations
                    .iter()
                    .position(|m| m.id == *current)
                    .ok_or("Database has invalid current migration")?,
            ),
            None => None,
        };

        Ok(current_index)
    }

    pub fn revisions_to_apply(
        &self,
        all_migrations: &[Migration],
        current: Option<&MigrationId>,
    ) -> Result<Vec<String>, &'static str> {
        let current_index = self.resolve_current_index(all_migrations, current)?;
        log::debug!("Current index: {:?}", current_index);

        let target_index = self.resolve_revision_index(all_migrations, current_index)?;
        log::debug!("Target index: {:?}", target_index);

        if let Some(current_index) = current_index {
            if target_index <= current_index {
                return Err("Revision is already applied");
            }
        }

        Ok(
            all_migrations[current_index.map(|c| c + 1).unwrap_or(0)..=target_index]
                .iter()
                .map(|m| m.stringify_id().clone())
                .collect(),
        )
    }

    pub fn revisions_to_revert(
        &self,
        all_migrations: &[Migration],
        current: Option<&MigrationId>,
    ) -> Result<Vec<String>, &'static str> {
        if current.is_none() {
            return Err("No current migration");
        }

        let current_index = self
            .resolve_current_index(all_migrations, current)?
            .unwrap();
        let target_index = self.resolve_revision_index(all_migrations, Some(current_index))?;

        if target_index >= current_index {
            return Err("Revision is already reverted");
        }

        Ok(all_migrations[target_index + 1..=current_index]
            .iter()
            .map(|m| m.stringify_id().clone())
            .collect())
    }
}

impl TryFrom<&str> for Revision {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let prepare = |s: &str| -> Result<(String, i32), &'static str> {
            let s = s.trim().to_lowercase();
            let items = s.splitn(2, ':').collect::<Vec<&str>>();
            let head_or_current = items[0];
            let relative_revision = match items.get(1) {
                Some(s) => s.parse::<i32>().map_err(|_| "Invalid relative revision")?,
                None => 0,
            };

            Ok((head_or_current.to_string(), relative_revision))
        };

        let (head_or_current, relative_revision) = prepare(value)?;

        match head_or_current.as_str() {
            "head" => Ok(Revision::Relative(RelativeRevision::Head(
                relative_revision,
            ))),
            "current" => Ok(Revision::Relative(RelativeRevision::Current(
                relative_revision,
            ))),
            _ => Ok(Revision::Absolute(value.to_string())),
        }
    }
}

impl FromStr for Revision {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Revision::try_from(s)
    }
}
