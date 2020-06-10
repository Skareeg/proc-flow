use proc_flow_lib as pf;
use pf::axiom::prelude::*;

#[test]
fn axiom_message_recv() {
    let as1 = ActorSystem::create(ActorSystemConfig::default());
    let as2 = ActorSystem::create(ActorSystemConfig::default());

    let tm1 = axiom::cluster::TcpClusterMgr::create(&as1, "127.0.0.1:7001".parse::<std::net::SocketAddr>().unwrap());
    let _tm2 = axiom::cluster::TcpClusterMgr::create(&as2, "127.0.0.1:7002".parse::<std::net::SocketAddr>().unwrap());

    use std::sync::*;

    let val = Arc::new(Mutex::new(0));

    match tm1.connect("127.0.0.1:7002".parse::<std::net::SocketAddr>().unwrap(), std::time::Duration::from_secs(5)) {
        Ok(()) => {
            let aid1 = as1.spawn().with(val.clone(), |state: Arc<Mutex<i32>>, ctx: Context, msg: Message| async move {
                match msg.content_as::<i32>() {
                    Some(v) => {
                        *state.lock().expect("mutex lock failed") = *v;
                        ctx.system.trigger_shutdown();
                    },
                    None => {}
                }
                Ok(Status::done(state))
            }).unwrap();
            let aid2 = as2.spawn().with(aid1.clone(), |state: Aid, ctx: Context, msg: Message| async move {
                match msg.content_as::<i32>() {
                    Some(v) => {
                        let _ = state.send_new((*v).clone());
                        ctx.system.trigger_shutdown();
                    },
                    None => {}
                }
                Ok(Status::done(state))
            }).unwrap();
            let _ = aid2.send_new(5 as i32);
        }
        Err(e) => {
            panic!("could not connect actor systems: {}", e.to_string());
        }
    }

    assert_eq!(ShutdownResult::Ok, as1.await_shutdown(std::time::Duration::from_secs(10)));
    assert_eq!(ShutdownResult::Ok, as2.await_shutdown(std::time::Duration::from_secs(10)));

    assert_eq!(5, *val.lock().unwrap());
}