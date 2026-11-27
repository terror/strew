use super::*;

pub(crate) fn run(config: &Config, dry_run: bool, verbose: bool) -> Result {
  let plan = Plan::new(config, true);

  let total = plan.items.len();

  let failures = plan
    .items
    .into_iter()
    .map(|item| {
      item.and_then(|actions| {
        actions.iter().try_for_each(|action| {
          perform(action, &config.base, dry_run, verbose)
        })
      })
    })
    .filter_map(Result::err)
    .inspect(print_error)
    .count();

  if failures > 0 {
    bail!("{failures} of {total} items failed");
  }

  Ok(())
}

fn perform(
  action: &Action,
  base: &Path,
  dry_run: bool,
  verbose: bool,
) -> Result {
  if !action.changes() {
    if verbose {
      action.print(dry_run);
    }

    return Ok(());
  }

  if dry_run {
    action.print(dry_run);
    return Ok(());
  }

  if let Action::Run(_) = action {
    action.print(dry_run);
    return action.execute(base);
  }

  action.execute(base)?;

  action.print(dry_run);

  Ok(())
}
