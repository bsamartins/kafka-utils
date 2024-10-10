use itertools::Itertools;

#[derive(Clone)]
pub struct Data {
    pub name: String,
    pub address: String,
    pub email: String,
}

impl Data {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn address(&self) -> &str {
        &self.address
    }

    pub(crate) fn email(&self) -> &str {
        &self.email
    }
}

pub fn generate_fake_names() -> Vec<Data> {
    use fakeit::{address, contact, name};

    (0..20)
        .map(|_| {
            let name = name::full();
            let address = format!(
                "{}\n{}, {} {}",
                address::street(),
                address::city(),
                address::state(),
                address::zip()
            );
            let email = contact::email();

            Data {
                name,
                address,
                email,
            }
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect_vec()
}
