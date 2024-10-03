awslocal kafka create-cluster \
--cluster-name "EventsCluster" \
--broker-node-group-info file://brokernodegroupinfo.json \
--kafka-version "2.8.0" \
--number-of-broker-nodes 3