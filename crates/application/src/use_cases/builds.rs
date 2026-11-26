//! Builds do Path of Building: listar os salvos e mostrar o que a avaliação entende de um deles.

use std::sync::Arc;

use mirrorsight_domain::evaluation::profile::BuildFile;
use mirrorsight_domain::evaluation::{BuildChoice, BuildDetails, GameKnowledge};
use mirrorsight_domain::ports::BuildRepository;

use crate::AppError;

pub struct BuildService {
    repo: Arc<dyn BuildRepository>,
    knowledge: Arc<dyn GameKnowledge>,
}

impl BuildService {
    pub fn new(repo: Arc<dyn BuildRepository>, knowledge: Arc<dyn GameKnowledge>) -> Self {
        Self { repo, knowledge }
    }

    pub fn list(&self) -> Result<Vec<BuildFile>, AppError> {
        Ok(self.repo.list()?)
    }

    /// Resumo do perfil (skill principal, foco) com a escolha de sets, e os sets disponíveis.
    pub fn details(&self, path: &str, choice: &BuildChoice) -> Result<BuildDetails, AppError> {
        let input = self.repo.load(path)?;
        Ok(input.details(choice, self.knowledge.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use mirrorsight_domain::evaluation::BuildInput;
    use mirrorsight_domain::ports::{PortError, PortResult};

    use super::*;

    struct OneBuild;
    impl BuildRepository for OneBuild {
        fn list(&self) -> PortResult<Vec<BuildFile>> {
            Ok(vec![BuildFile { name: "Fireball".into(), path: "f.xml".into(), folder: None }])
        }
        fn load(&self, path: &str) -> PortResult<BuildInput> {
            match path {
                "f.xml" => Ok(BuildInput {
                    name: "Fireball".into(),
                    ascendancy: Some("Elementalist".into()),
                    main_skill_gems: vec!["Fireball".into()],
                    ..Default::default()
                }),
                _ => Err(PortError::NotFound(path.into())),
            }
        }
    }

    #[test]
    fn lists_and_summarizes() {
        let svc = BuildService::new(Arc::new(OneBuild), Arc::new(mirrorsight_gamedata::EmbeddedKnowledge::poe1()));
        assert_eq!(svc.list().unwrap().len(), 1);
        let s = svc.details("f.xml", &BuildChoice::default()).unwrap().summary;
        assert_eq!(s.main_skill.as_deref(), Some("Fireball"));
        assert_eq!(s.class.as_deref(), Some("Elementalist"));
        assert!(s.focus.contains(&"caster".to_owned()));
        assert!(svc.details("x.xml", &BuildChoice::default()).is_err());
    }
}
