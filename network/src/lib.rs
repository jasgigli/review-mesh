use libp2p::{
    identity, PeerId, Swarm, mdns::{Mdns, MdnsConfig}, noise::{Keypair as NoiseKeypair, NoiseConfig, X25519Spec, AuthenticKeypair, AuthenticKeypairRef},
    floodsub::{self, Floodsub, FloodsubEvent, Topic}, swarm::SwarmEvent, core::upgrade, tcp::TcpConfig, Transport, NetworkBehaviour, Multiaddr
};
use base58::{ToBase58, FromBase58};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use common::{ReviewSession, Comment, ChatLine};
use std::collections::HashSet;
use std::error::Error;
use std::task::{Context, Poll};
use futures::prelude::*;

#[derive(NetworkBehaviour)]
pub struct ReviewMeshBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
}

pub struct Network {
    pub peer_id: PeerId,
    pub swarm: Swarm<ReviewMeshBehaviour>,
    pub topics: HashSet<Topic>,
}

impl Network {
    pub async fn new(session: &ReviewSession, secret: &[u8]) -> Result<Self, Box<dyn Error>> {
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        let noise_keys = NoiseKeypair::<X25519Spec>::new().into_authentic(&id_keys)?;
        let transport = TcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(libp2p::yamux::YamuxConfig::default())
            .boxed();

        let mut floodsub = Floodsub::new(peer_id.clone());
        let mdns = Mdns::new(MdnsConfig::default()).await?;
        let behaviour = ReviewMeshBehaviour { floodsub, mdns };
        let mut swarm = Swarm::new(transport, behaviour, peer_id.clone());
        Ok(Self {
            peer_id,
            swarm,
            topics: HashSet::new(),
        })
    }

    pub fn join_topic(&mut self, topic: &str) {
        let t = Topic::new(topic);
        self.swarm.behaviour_mut().floodsub.subscribe(t.clone());
        self.topics.insert(t);
    }

    pub fn send_message(&mut self, topic: &str, data: &[u8]) {
        let t = Topic::new(topic);
        self.swarm.behaviour_mut().floodsub.publish(t, data);
    }

    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<(String, Vec<u8>)>> {
        loop {
            match self.swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(SwarmEvent::Behaviour(ReviewMeshBehaviourEvent::Floodsub(FloodsubEvent::Message(msg))))) => {
                    let topic = msg.topics.get(0).map(|t| t.id().clone()).unwrap_or_default();
                    return Poll::Ready(Some((topic, msg.data.clone())));
                }
                Poll::Ready(Some(_)) => continue,
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
            }
        }
    }

    pub fn generate_invite_token(session_id: &str, secret: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
        mac.update(session_id.as_bytes());
        let result = mac.finalize().into_bytes();
        let mut token = session_id.as_bytes().to_vec();
        token.extend(&result);
        token.to_base58()
    }

    pub fn parse_invite_token(token: &str, secret: &[u8]) -> Option<String> {
        let data = token.from_base58().ok()?;
        if data.len() < 32 { return None; }
        let (session_id, mac_bytes) = data.split_at(data.len() - 32);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).ok()?;
        mac.update(session_id);
        if mac.verify_slice(mac_bytes).is_ok() {
            Some(String::from_utf8_lossy(session_id).to_string())
        } else {
            None
        }
    }
}
