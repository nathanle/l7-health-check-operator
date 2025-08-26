extern crate serde_derive;
use kube::{api::{Api, ListParams, WatchEvent}, client::Client};
use k8s_openapi::api::core::v1::Node;
use futures::{StreamExt, TryStreamExt};
use serde::{Serialize, Deserialize};
use kube_derive::CustomResource;
use schemars::JsonSchema;

#[derive(CustomResource, Serialize, Deserialize, Default, Clone, Debug, JsonSchema)]
#[kube(group = "health-check.linode.com", version = "v1", kind="NodeCheck", namespaced)]
#[allow(non_snake_case)]
pub struct MemberSpec {
  pub memberOf: Option<String>
}

#[tokio::main]
async fn main() {
    println!("starting hello world operator");
    let client = Client::try_default().await.expect("cannot infer client config");
    let client2 = Client::try_default().await;

    //let crds: Api<Member> = Api::namespaced(client, "default");
    let nodecheck: Api<NodeCheck> = Api::namespaced(client, "default");
    let nds: Api<Node> = Api::all(client2.expect("Node client connection failed"));
    let mut watcher = nds.watch(&Default::default(), "0").await.unwrap().boxed();
    while let Some(event) = watcher.try_next().await.expect("failed") {
        match event {
            WatchEvent::Added(node) => {
                println!("Node Added: {}", node.metadata.name.unwrap_or_default());
            }
            WatchEvent::Modified(_node) => {
            },
            WatchEvent::Deleted(node) => {
              println!("Deleted {}", node.metadata.name.unwrap());
            },
            WatchEvent::Error(member) => println!("error: {}", member),
            _ => {}
        }
    }

    let lp = ListParams::default();
    println!("{:#?}", &lp);

    println!("subscribing events of type health-check.linode.com/v1");
    let mut stream = nodecheck.watch(&lp, "0").await.unwrap().boxed();
    while let Some(status) = stream.try_next().await.expect("watch stream failed") {
        match status {
            WatchEvent::Added(node) => {
                println!("Node applied: {}", node.metadata.name.unwrap_or_default());
              //match member.spec.memberOf {
              //  None => println!("welcome {}",member.metadata.name.unwrap()),
              //  Some(member_of) => println!("welcome {} to the team {}"
              //            ,member.metadata.name.unwrap()
              //            ,member_of),
              //}
            },
            WatchEvent::Modified(_member) => {
            },
            WatchEvent::Deleted(member) => {
              println!("sad to see you go {}",member.metadata.name.unwrap());
            },
            WatchEvent::Error(member) => println!("error: {}", member),
            _ => {}
        }
    }
    println!("done");
}
