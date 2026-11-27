use super::*;

pub(crate) fn run(config: &Config) -> Result {
  let plan = Plan::new(config, false);

  let mut failures = 0;

  for item in &plan.items {
    match item {
      Ok(actions) => {
        for action in actions {
          action.print(true);
        }

        if actions.iter().any(Action::changes) {
          failures += 1;
        }
      }
      Err(error) => {
        print_error(error);
        failures += 1;
      }
    }
  }

  if failures > 0 {
    bail!("{failures} of {} items need attention", plan.items.len());
  }

  Ok(())
}
