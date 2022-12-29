use actix::prelude::*;

use crate::messages::*;

pub struct Determinator {
    worker: Recipient<Job>
}

impl Determinator {
    pub fn start(worker: Recipient<Job>) -> Addr<Self> {
        Determinator::create(|_ctx| {
            Determinator { worker }
        })
    }
}

impl Actor for Determinator {
    type Context = Context<Self>;
}

impl Handler<Activity> for Determinator {
    type Result = ();

    fn handle(&mut self, _msg: Activity, _ctx: &mut Self::Context) {
        self.worker.try_send(Job).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Message, Debug, PartialEq, Clone)]
    #[rtype(result = "Vec<Job>")]
    pub struct Assert;

    #[derive(Clone, Default)]
    struct TestProbe {
        received: Vec<Job>
    }
    impl Actor for TestProbe {
        type Context = Context<Self>;
    }
    impl Handler<Job> for TestProbe {
        type Result = ();

        fn handle(&mut self, msg: Job, _ctx: &mut Self::Context) -> Self::Result {
            self.received.push(msg);
        }
    }
    impl Handler<Assert> for TestProbe {
        type Result = Vec<Job>;

        fn handle(&mut self, _msg: Assert, _ctx: &mut Self::Context) -> Self::Result {
            self.received.clone()
        }
    }

    #[actix_rt::test]
    async fn test_that_determinator_handles_activity() {
        let worker_addr = TestProbe::default().start();

        let sut = Determinator::start(worker_addr.clone().recipient());
        sut.send(Activity).await.unwrap();

        let received = worker_addr.send(Assert).await.unwrap();

        assert_eq!(vec![Job], received);
    }
}
