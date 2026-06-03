use crate::{
    git::GitWrapper,
    logic::{
        errors::PipelineError,
        pipeline::{Pipeline, Precondition, Step},
    },
};
use tokio::sync::mpsc::Sender;

pub async fn release_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("release/{name}");

    let send = async |msg: &str| {
        let _ = sender.send(msg.into()).await;
    };

    send(&format!("Starting creation of release {name}...")).await;

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresMissingBranch(branch.clone()))
        .step(Step::Checkout("develop".into()))
        .step(Step::Pull())
        .step(Step::CreateBranch(branch.clone()))
        .run()
        .await?;

    send("Release started successfully").await;

    Ok(())
}

pub async fn release_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("release/{name}");

    let send = async |msg: &str| {
        let _ = sender.send(msg.into()).await;
    };

    send(&format!("Finishing release {name}...")).await;

    let main_branch = GitWrapper::get_main_branch().await;

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresExistingLocalBranch(branch.clone()))
        .step(Step::Checkout(branch.clone()))
        .step(Step::Pull())
        .step(Step::Push())
        .step(Step::Checkout(main_branch.clone()))
        .step(Step::Pull())
        .step(Step::Merge(branch.clone()))
        .step(Step::Tag(name.to_string()))
        .step(Step::Push())
        .step(Step::PushTags())
        .step(Step::Checkout("develop".into()))
        .step(Step::Merge(main_branch.clone()))
        .step(Step::Push())
        .step(Step::DeleteBranch(branch.clone()))
        .run()
        .await?;

    send("Release finished successfully").await;

    Ok(())
}
