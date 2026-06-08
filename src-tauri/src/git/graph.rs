use super::types::{CommitInfo, GraphLine};

const COLORS: u32 = 6;

/// Назначает каждому коммиту колонку (`column`) и набор линий (`lines`)
/// для отрисовки lane-графа. Коммиты ожидаются в порядке `git log`
/// (новейшие первыми, топологически согласовано).
pub fn assign_lanes(commits: &mut [CommitInfo]) {
    let mut lanes: Vec<Option<String>> = Vec::new();

    for idx in 0..commits.len() {
        let oid = commits[idx].oid.clone();
        let parents = commits[idx].parents.clone();
        let mut lines: Vec<GraphLine> = Vec::new();

        // 1. колонка узла. `lane_was_active` = lane уже указывал на этот коммит
        // (тянулась сверху); иначе lane появляется здесь впервые — над dot
        // ничего нет, и его вертикаль рисуется только от dot вниз (`tip`),
        // иначе получится «палка в воздух».
        let (col, lane_was_active) = match lanes
            .iter()
            .position(|l| l.as_deref() == Some(oid.as_str()))
        {
            Some(c) => (c, true),
            None => match lanes.iter().position(|l| l.is_none()) {
                Some(c) => (c, false),
                None => {
                    lanes.push(None);
                    (lanes.len() - 1, false)
                }
            },
        };

        // 2. входящие мержи: другие lane, указывающие на oid
        for i in 0..lanes.len() {
            if i != col && lanes[i].as_deref() == Some(oid.as_str()) {
                let style = if i > col { "merge-left" } else { "merge-right" };
                lines.push(GraphLine {
                    from_column: i as u32,
                    to_column: col as u32,
                    color: (i as u32) % COLORS,
                    style: style.to_string(),
                });
                lanes[i] = None;
            }
        }

        // 3. узел разрешён
        lanes[col] = None;

        // 4. сквозные lane
        for i in 0..lanes.len() {
            if i != col && lanes[i].is_some() {
                lines.push(GraphLine {
                    from_column: i as u32,
                    to_column: i as u32,
                    color: (i as u32) % COLORS,
                    style: "straight".to_string(),
                });
            }
        }

        // 5. вертикаль узла (всегда). Если lane появился здесь впервые,
        // рисуем только нижнюю половину (`tip`).
        lines.push(GraphLine {
            from_column: col as u32,
            to_column: col as u32,
            color: (col as u32) % COLORS,
            style: if lane_was_active { "straight" } else { "tip" }.to_string(),
        });

        // исходящие родители
        if let Some((first, rest)) = parents.split_first() {
            lanes[col] = Some(first.clone());
            for p in rest {
                if lanes.iter().any(|l| l.as_deref() == Some(p.as_str())) {
                    continue;
                }
                let j = match lanes.iter().position(|l| l.is_none()) {
                    Some(c) => c,
                    None => {
                        lanes.push(None);
                        lanes.len() - 1
                    }
                };
                lanes[j] = Some(p.clone());
                lines.push(GraphLine {
                    from_column: col as u32,
                    to_column: j as u32,
                    color: (j as u32) % COLORS,
                    style: "fork".to_string(),
                });
            }
        }
        // корень (нет родителей): lanes[col] остаётся None

        commits[idx].column = col as u32;
        commits[idx].lines = lines;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::types::CommitInfo;

    fn c(oid: &str, parents: &[&str]) -> CommitInfo {
        CommitInfo {
            oid: oid.to_string(),
            short_oid: oid.to_string(),
            message: String::new(),
            author: String::new(),
            author_email: String::new(),
            date: String::new(),
            parents: parents.iter().map(|s| s.to_string()).collect(),
            refs: Vec::new(),
            column: 0,
            lines: Vec::new(),
            unpushed: false,
        }
    }

    #[test]
    fn linear_history_single_column() {
        let mut v = vec![c("C", &["B"]), c("B", &["A"]), c("A", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0);
        assert_eq!(v[1].column, 0);
        assert_eq!(v[2].column, 0);
        // Самый верхний коммит — tip (его lane появляется здесь), остальные —
        // straight (lane тянется сверху).
        assert!(v[0]
            .lines
            .iter()
            .any(|l| l.style == "tip" && l.from_column == 0 && l.to_column == 0));
        for row in &v[1..] {
            assert!(row
                .lines
                .iter()
                .any(|l| l.style == "straight" && l.from_column == 0 && l.to_column == 0));
        }
    }

    #[test]
    fn branch_and_merge() {
        // M — мерж (parents P1, P2); P1, P2 → Base
        let mut v = vec![
            c("M", &["P1", "P2"]),
            c("P1", &["Base"]),
            c("P2", &["Base"]),
            c("Base", &[]),
        ];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0); // M
        assert!(v[0]
            .lines
            .iter()
            .any(|l| l.style == "fork" && l.from_column == 0 && l.to_column == 1));
        assert_eq!(v[2].column, 1); // P2 на lane 1
        assert_eq!(v[3].column, 0); // Base
        assert!(v[3]
            .lines
            .iter()
            .any(|l| l.style == "merge-left" && l.from_column == 1 && l.to_column == 0));
    }

    #[test]
    fn root_commit_no_fork() {
        let mut v = vec![c("R", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0);
        assert!(!v[0].lines.iter().any(|l| l.style == "fork"));
        // Единственный коммит — он же tip, lane не тянется сверху.
        assert!(v[0]
            .lines
            .iter()
            .any(|l| l.style == "tip" && l.from_column == 0));
    }

    #[test]
    fn unreferenced_tip_gets_new_lane() {
        // A держит lane0 занятым (родитель B), затем независимый C → lane1
        let mut v = vec![c("A", &["B"]), c("C", &["D"]), c("B", &[]), c("D", &[])];
        assign_lanes(&mut v);
        assert_eq!(v[0].column, 0); // A
        assert_eq!(v[1].column, 1); // C — lane0 занят B

        // C появляется в lane1 впервые → его вертикаль — tip (только нижняя
        // половина), иначе во фронте получим «палку» из dot вверх в воздух.
        let c_own_line = v[1]
            .lines
            .iter()
            .find(|l| l.from_column == 1 && l.to_column == 1)
            .expect("у C должна быть собственная вертикаль");
        assert_eq!(c_own_line.style, "tip");

        // A — самый первый коммит, его lane тоже tip.
        let a_own_line = v[0]
            .lines
            .iter()
            .find(|l| l.from_column == 0 && l.to_column == 0)
            .expect("у A должна быть собственная вертикаль");
        assert_eq!(a_own_line.style, "tip");

        // B берёт lane0 как продолжение A → straight, не tip.
        let b_own_line = v[2]
            .lines
            .iter()
            .find(|l| l.from_column == 0 && l.to_column == 0)
            .expect("у B должна быть собственная вертикаль");
        assert_eq!(b_own_line.style, "straight");
    }
}
