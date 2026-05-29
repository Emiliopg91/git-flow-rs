use std::sync::mpsc::Sender;

use super::{
    errors::PipelineError,
    pipeline::{Pipeline, Precondition, Step},
};

pub async fn bugfix_start(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Starting creation of bugfix {}...", name));

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresMissingBranch(branch.clone()))
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::CreateBranch(branch))
        .run()
        .await?;

    send("Bugfix started succesfully");

    Ok(())
}

pub async fn bugfix_finish(name: &str, sender: Sender<String>) -> Result<(), PipelineError> {
    let branch = format!("bugfix/{}", name);
    let send = |msg: &str| {
        let _ = sender.send(msg.into());
    };

    send(&format!("Finishing bugfix {}...", name));

    Pipeline::new(sender.clone())
        .requires(Precondition::RequiresExistingLocalBranch(branch.clone()))
        .step(Step::Checkout(branch.clone()))
        .step(Step::Pull())
        .step(Step::Push())
        .step(Step::Checkout("develop".to_string()))
        .step(Step::Pull())
        .step(Step::Merge(branch.clone()))
        .step(Step::Commit(format!("Merge after {} bugfix merge", name)))
        .step(Step::Push())
        .step(Step::DeleteBranch(branch.clone()))
        .run()
        .await?;

    send("Bugfix finished succesfully");

    Ok(())
}
