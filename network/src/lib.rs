use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio;

use libp2p::{
    core::upgrade,
    identity,
    noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp,
    yamux,
    PeerId, Transport,
};
use libp2p::futures::StreamExt;
use libp2p_mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig};
use libp2p::floodsub::{self, Floodsub, FloodsubEvent, Topic};
use libp2p::swarm::NetworkBehaviour;
use common::{ReviewSession, Comment};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ReviewMeshBehaviourEvent")]
pub struct ReviewMeshBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
}

#[allow(clippy::large_enum_variant)]
pub enum ReviewMeshBehaviourEvent {
    Floodsub(FloodsubEvent),
    Mdns(libp2p_mdns::Event),
}

impl From<FloodsubEvent> for ReviewMeshBehaviourEvent {
    fn from(event: FloodsubEvent) -> Self {
        ReviewMeshBehaviourEvent::Floodsub(event)
    }
}

impl From<libp2p_mdns::Event> for ReviewMeshBehaviourEvent {
    fn from(event: libp2p_mdns::Event) -> Self {
        ReviewMeshBehaviourEvent::Mdns(event)
    }
}

pub struct NetworkManager {
    pub swarm: libp2p::Swarm<ReviewMeshBehaviour>,
    topic: Topic,
}

impl NetworkManager {
    pub fn new(secret_key_seed: Option<u8>) -> Result<Self, Box<dyn Error>> {
        let id_keys = match secret_key_seed {
            Some(seed) => {
                let mut sk_bytes = [0u8; 32];
                sk_bytes[0] = seed;
                identity::Keypair::ed25519_from_bytes(&mut sk_bytes).unwrap()
            }
            None => identity::Keypair::generate_ed25519(),
        };
        let peer_id = PeerId::from(id_keys.public());

        let noise_config = noise::Config::new(&id_keys).unwrap();

        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise_config)
            .multiplex(yamux::Config::default())
            .timeout(std::time::Duration::from_secs(20))
            .boxed();

        let topic = floodsub::Topic::new("reviews");

        let mut swarm = {
            let mdns = Mdns::new(MdnsConfig::default(), peer_id)?;
            let mut behaviour = ReviewMeshBehaviour {
                floodsub: Floodsub::new(peer_id),
                mdns,
            };
            behaviour.floodsub.subscribe(topic.clone());
            SwarmBuilder::with_executor(transport, behaviour, peer_id, Box::new(|fut| { tokio::spawn(fut); })).build()
        };

        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self { swarm, topic })
    }

    pub fn publish_review_session(&mut self, review: &ReviewSession) {
        let review_json = serde_json::to_string(review).unwrap();
        self.swarm.behaviour_mut().floodsub.publish(self.topic.clone(), review_json.as_bytes());
    }

    pub fn publish_comment(&mut self, comment: &Comment) {
        let comment_json = serde_json::to_string(comment).unwrap();
        self.swarm.behaviour_mut().floodsub.publish(self.topic.clone(), comment_json.as_bytes());
    }

    pub fn get_known_peers(&self) -> HashSet<PeerId> {
        self.swarm.behaviour().mdns.discovered_nodes().cloned().collect()
    }
}

impl Future for NetworkManager {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => match event {
                    SwarmEvent::Behaviour(ReviewMeshBehaviourEvent::Floodsub(floodsub_event)) => {
                        if let floodsub::FloodsubEvent::Message(message) = floodsub_event {
                            println!(
                                "Received: '{:?}' from {:?}",
                                String::from_utf8_lossy(&message.data),
                                message.source
                            );
                        }
                    }
                    SwarmEvent::Behaviour(ReviewMeshBehaviourEvent::Mdns(mdns_event)) => {
                        match mdns_event {
                            libp2p_mdns::Event::Discovered(list) => {
                                for (peer, _) in list {
                                    self.swarm.behaviour_mut().floodsub.add_node_to_partial_view(peer);
                                }
                            }
                            libp2p_mdns::Event::Expired(list) => {
                                for (peer, _) in list {
                                    if !self.swarm.behaviour().mdns.has_node(&peer) {
                                        self.swarm.behaviour_mut().floodsub.remove_node_from_partial_view(&peer);
                                    }
                                }
                            }
                        }
                    }
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    _ => {}
                },
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
