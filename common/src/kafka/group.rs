use crate::kafka::client::{create_admin_client, create_base_client, Config};
use crate::kafka::types::ListConsumerGroupEntry;
use rdkafka::admin::{AdminOptions, GroupResult};
use rdkafka::consumer::Consumer;
use rdkafka::groups::GroupInfo;

pub fn list(config: &Config, consumer_group: Option<String>) -> Vec<ListConsumerGroupEntry> {
    let result = create_base_client(config)
        .fetch_group_list(None, config.timeout)
        .expect("could not fetch group list");

    let mut groups: Vec<ListConsumerGroupEntry> = result.groups()
        .iter()
        .filter(|g| filter_group(g, consumer_group.clone()))
        .map(|g|
            ListConsumerGroupEntry {
                name: g.name().to_string(),
                state: g.state().to_string(),
            }
        ).collect();
    groups.sort_by_key(|i| i.name.clone());
    groups
}

fn filter_group(group: &GroupInfo, group_query: Option<String>) -> bool {
    match group_query.clone() {
        Some(q) => group.name().starts_with(q.as_str()),
        None => true,
    }
}

pub async fn delete(config: &Config, consumer_group: Option<String>) -> Vec<GroupResult> {
    let result = create_base_client(config)
        .fetch_group_list(None, config.timeout)
        .expect("could not fetch group list");

    let mut groups = result.groups()
        .iter()
        .filter(|g| filter_group(g, consumer_group.clone()))
        .collect::<Vec<_>>();
    groups.sort_by_key(|i| i.name());

    let groups_to_delete: Vec<&str>  = groups.iter().map(|g| g.name())
        .collect();

    create_admin_client(&config)
        .delete_groups(&groups_to_delete, &AdminOptions::new())
        .await
        .expect("could not delete groups")
}