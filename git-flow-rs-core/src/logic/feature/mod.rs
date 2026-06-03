use super::{
    errors::PipelineError,
    pipeline::{Pipeline, Precondition, Step},
};
use tokio::sync::mpsc::Sender;

pub async fn feature_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of feature {}...", name));

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresMissingBranch(branch.clone()))
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::CreateBranch(branch))
        .run()
        .await?;

    send("Feature started succesfully");

    Ok(())
}

pub async fn feature_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("feature/{}", name);

    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing feature {}...", name));

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresExistingLocalBranch(branch.clone()))
        .step(Step::Checkout(branch.clone()))
        .step(Step::Pull())
        .step(Step::Push())
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::Merge(branch.clone()))
        .step(Step::Commit(format!("Merge after {} feature merge", name)))
        .step(Step::Push())
        .step(Step::DeleteBranch(branch.clone()))
        .run()
        .await?;

    send("Feature finished succesfully");

    Ok(())
}
