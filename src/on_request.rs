pub struct Req {}
pub struct Res {}

pub enum Error {
    AllWorkerBusy,
}

type Result<T> = std::result::Result<T, Error>;

pub async fn on_request(req: &mut Req, res: &mut Res) -> Result<()> {
    for _ in 0..2 {
        match find_proper_worker().await? {
            FindWorkerResult::Me => return execute_on_me(req, res).await,
            FindWorkerResult::Other { address } => {
                return execute_on_other(address, req, res).await;
            }
            FindWorkerResult::Nobody => {
                tokio::time::sleep(tokio::time::Duration::from_millis(14)).await;
            }
        }
    }

    Err(Error::AllWorkerBusy)
}

async fn execute_on_me(req: &mut Req, res: &mut Res) -> Result<()> {
    todo!()
}

async fn execute_on_other(address: Address, req: &mut Req, res: &mut Res) -> Result<()> {
    todo!()
}

enum FindWorkerResult {
    Me,
    Other { address: Address },
    Nobody,
}

struct Address {}

async fn find_proper_worker() -> Result<FindWorkerResult> {
    todo!()
}
