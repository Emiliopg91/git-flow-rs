use crate::{
    git::GitWrapper,
    logic::{
        errors::PipelineError,
        pipeline::{Pipeline, Precondition, Step},
    },
};
use tokio::sync::mpsc::Sender;

pub async fn hotfix_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("hotfix/{}", name);
    let main_branch = GitWrapper::get_main_branch().await;

    let send = async |msg: &str| {
        let _ = sender.send(msg.into()).await;
    };

    send(&format!("Starting creation of hotfix {}...", name)).await;

    Pipeline::new(sender.clone())
        .step(Step::Checkout(main_branch.clone()))
        .step(Step::Pull())
        .step(Step::CreateBranch(branch.clone()))
        .run()
        .await?;

    send("Hotfix started succesfully").await;

    Ok(())
}

pub async fn hotfix_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("hotfix/{}", name);
    let main_branch = GitWrapper::get_main_branch().await;

    let send = async |msg: &str| {
        let _ = sender.send(msg.into()).await;
    };

    send(&format!("Finishing hotfix {}...", name)).await;

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresExistingLocalBranch(branch.clone()))
        .step(Step::Checkout(branch.clone()))
        .step(Step::Pull())
        .step(Step::Push())
        .step(Step::Checkout(main_branch.clone()))
        .step(Step::Pull())
        .step(Step::Merge(branch.clone()))
        .step(Step::Commit(format!("Merge after {} hotfix merge", name)))
        .step(Step::Push())
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::Merge(main_branch.clone()))
        .step(Step::Push())
        .step(Step::DeleteBranch(branch.clone()))
        .run()
        .await?;

    send("Hotfix finished succesfully").await;

    Ok(())
}
