use cyw43::NetDriver;
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_net::{Config, Stack, StackResources};
use embassy_time::Instant;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use static_cell::StaticCell;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<NetDriver<'static>>) -> ! {
    stack.run().await
}

static CYW43_NET_STACK: StaticCell<Stack<NetDriver>> = StaticCell::new();
static CYW43_NET_RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();

pub async fn init(spawner: Spawner, net_device: NetDriver<'static>) -> &Stack<NetDriver<'static>> {
    let mut rng = SmallRng::seed_from_u64(Instant::now().as_micros());

    // allocate memory space for the network stack, with 2 sockets
    let resources = CYW43_NET_RESOURCES.init(StackResources::<2>::new());

    // utilize dhcp to acquire an IP
    let config = Config::dhcpv4(Default::default());

    let stack = CYW43_NET_STACK.init(Stack::new(net_device, config, resources, rng.next_u64()));

    // run the network stack task
    unwrap!(spawner.spawn(net_task(stack)));

    return stack;
}
