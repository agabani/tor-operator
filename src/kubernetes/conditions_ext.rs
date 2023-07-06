use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;

pub trait ConditionsExt {
    fn merge_from(&self, other: &Self) -> Self;
}

impl ConditionsExt for Vec<Condition> {
    fn merge_from(&self, other: &Self) -> Self {
        let mut results: Vec<_> = self
            .iter()
            .map(|s| match other.iter().find(|o| s.type_ == o.type_) {
                Some(o) => {
                    if s.status == o.status && s.reason == o.reason && s.message == o.message {
                        s.clone()
                    } else {
                        o.clone()
                    }
                }
                None => s.clone(),
            })
            .collect();

        for o in other {
            if !results.iter().any(|s| s.type_ == o.type_) {
                results.push(o.clone());
            }
        }

        results
    }
}
