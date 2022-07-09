use rscl::{core::*, context::{SimpleContext, Global}};
use rscl_proc::global_context;

#[global_context]
static CONTEXT : SimpleContext = SimpleContext::default();

static PROGRAM : &str = "void kernel add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        out[id] = in[id] + rhs[id];
    }
}";

#[test]
fn program () -> Result<()> {
    println!("{}", core::mem::size_of::<Device>());
    println!("{}", core::mem::size_of::<Option<Device>>());

    println!("{}", core::mem::size_of::<bool>());
    println!("{}", core::mem::size_of::<Option<bool>>());

    let dev = Device::first().unwrap();
    println!("{:?}", Global.num_devices());
    Ok(())
}

#[cfg(feature = "futures")]
#[test]
fn flag () {
    use rscl::event::EventStatus;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().unwrap()
        .block_on(async move {
            use std::time::Duration;
            use rscl::{event::FlagEvent, prelude::Event};

            let event : FlagEvent = FlagEvent::new().unwrap();
            let event2 = event.clone();
            let event3 = event.clone();

            let print = async move {
                while event3.status().unwrap() != EventStatus::Complete {
                    println!("Not done yet");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            };

            let complete = async move {
                event2.raw().wait_async().unwrap().await.unwrap();
            };

            let wait = async move {
                tokio::time::sleep(Duration::from_secs(5)).await;
                event.set_complete(None).unwrap();
            };

            tokio::join!(complete, wait, print);
        });
}