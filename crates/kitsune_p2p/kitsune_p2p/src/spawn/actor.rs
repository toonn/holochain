use crate::{actor, actor::*, event::*, types::*};
use futures::future::FutureExt;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

mod space;
use space::*;

ghost_actor::ghost_chan! {
    pub(crate) chan Internal<crate::KitsuneP2pError> {
        /// temp because ghost_chan doesn't allow empty Api
        fn ping() -> ();
    }
}

pub(crate) struct KitsuneP2pActor {
    #[allow(dead_code)]
    internal_sender: KitsuneP2pInternalSender<Internal>,
    #[allow(dead_code)]
    evt_sender: futures::channel::mpsc::Sender<KitsuneP2pEvent>,
    spaces: HashMap<Arc<KitsuneSpace>, Space>,
}

impl KitsuneP2pActor {
    pub fn new(
        internal_sender: KitsuneP2pInternalSender<Internal>,
        evt_sender: futures::channel::mpsc::Sender<KitsuneP2pEvent>,
    ) -> KitsuneP2pResult<Self> {
        Ok(Self {
            internal_sender,
            evt_sender,
            spaces: HashMap::new(),
        })
    }
}

impl KitsuneP2pHandler<(), Internal> for KitsuneP2pActor {
    fn handle_join(&mut self, input: actor::Join) -> KitsuneP2pHandlerResult<()> {
        let actor::Join { space, agent } = input;
        let space = Arc::new(space);
        let agent = Arc::new(agent);
        let space = match self.spaces.entry(space) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Space::new(self.internal_sender.clone())),
        };
        space.handle_join(agent)
    }

    fn handle_leave(&mut self, _input: actor::Leave) -> KitsuneP2pHandlerResult<()> {
        Ok(async move { Ok(()) }.boxed().into())
    }

    fn handle_request(&mut self, _input: actor::Request) -> KitsuneP2pHandlerResult<Vec<u8>> {
        Ok(async move { Ok(vec![]) }.boxed().into())
    }

    fn handle_broadcast(&mut self, _input: actor::Broadcast) -> KitsuneP2pHandlerResult<u32> {
        Ok(async move { Ok(0) }.boxed().into())
    }

    fn handle_multi_request(
        &mut self,
        _input: actor::MultiRequest,
    ) -> KitsuneP2pHandlerResult<Vec<actor::MultiRequestResponse>> {
        Ok(async move { Ok(vec![]) }.boxed().into())
    }
}