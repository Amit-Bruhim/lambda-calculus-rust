use std::collections::HashSet;
use crate::parser::Term;
use crate::utils;

pub fn fv(t: &Term) -> HashSet<String> {
    match t {
        Term::Variable(x) => {
            let mut set = HashSet::new();
            set.insert(x.clone());
            set
        }

        Term::Abstraction(x, body) => {
            let mut set = fv(body);
            set.remove(x);
            set
        }

        Term::Application(t1, t2) => {
            let mut set = fv(t1);
            let other = fv(t2);
            set.extend(other);
            set
        }
    }
}



/// Substitute: replace occurrences of `x` with `t1` inside `t2`.
/// Notation: t2 [x := t1] 
fn substitute(x: &str, t1: &Term, t2: &Term) -> Term {
    match t2{
        // Case 1: The term is a variable
        Term::Variable(name) => {
            if name == x {
                // If the name matches x, replace it with t1
                t1.clone()
            } else {
                // If it's a different variable, leave it unchanged
                t2.clone()
            }
        }, 
        // Case 2: The term is an application
        Term::Application(left, right) => {
            // Recursively substitute in both sides of the application
            Term::Application(
                Box::new(substitute(x, t1, left)),
                Box::new(substitute(x, t1, right)),
            )
        },
        // Case 3: The term is an abstraction (lambda)
        Term::Abstraction(y, body) => {
            let t1_fv = fv(t1);
            if y == x {
                Term::Abstraction(y.clone(), body.clone())
            }
            else if !t1_fv.contains(y) {
                Term::Abstraction(y.clone(), Box::new(substitute(x, t1, body)))
            } else {
                let mut used_vars = fv(body);
                used_vars.extend(t1_fv);
                used_vars.insert(x.to_string());
                let z = utils::fresh_var(&used_vars);
                let renamed_body = substitute(y, &Term::Variable(z.clone()), body);
                Term::Abstraction(z, Box::new(substitute(x, t1, &renamed_body)))
            }
        }
    }
}

fn is_value(t: &Term) -> bool {
    matches!(t, Term::Abstraction(_, _))
}

/// Call-by-Value reducer.
/// Returns Some(reduced_term) or None if no reduction is possible.
pub fn reduce_cbv(t: &Term) -> Option<Term> {
    match t {
        Term::Variable(_) => None,

        Term::Abstraction(_, _) => None,

        Term::Application(t1, t2) => {

            // E-App1
            if let Some(t1_reduced) = reduce_cbv(t1) {
                return Some(
                    Term::Application(
                        Box::new(t1_reduced),
                        Box::new((**t2).clone())
                    )
                );
            }

            // E-App2
            if is_value(t1) {
                if let Some(t2_reduced) = reduce_cbv(t2) {
                    return Some(
                        Term::Application(
                            Box::new((**t1).clone()),
                            Box::new(t2_reduced)
                        )
                    );
                }
            }

            // E-AppAbs
            if let Term::Abstraction(x, body) = &**t1 {
                if is_value(t2) {
                    return Some(substitute(x, t2, body));
                }
            }

            None
        }
    }
}

/// Call-by-Name reducer.
/// Returns Some(reduced_term) or None if no reduction is possible.
pub fn reduce_cbn(t: &Term) -> Option<Term> {
    match t {
        Term::Application(t1, t2) => {
            if let Term::Abstraction(x, body) = &**t1 {
                Some(substitute(x, t2, body))
            }
            else if let Some(t1_prime) = reduce_cbn(t1) {
                Some(Term::Application(Box::new(t1_prime), t2.clone()))
            } else{
                None
            }
        },
        _ => None,
    }
}