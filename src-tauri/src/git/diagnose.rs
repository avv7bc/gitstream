//! Распознавание частых git-проблем и предложение безопасных решений.
//!
//! Модель: `diagnose_sync` инспектирует локальное состояние (расхождение с
//! remote-tracking веткой) и возвращает `Situation` с набором `Remedy`. Весь
//! текст (заголовок/описание/подписи кнопок) локализуется на фронте по `id` —
//! backend отдаёт только идентификаторы, числа и уровень опасности.

use std::path::Path;

use serde::Serialize;

use super::query::run_git;

/// Одно предлагаемое решение. `id` определяет, какое действие выполнит фронт
/// (см. `useSync.applyRemedy`): "push_force_lease" | "pull_rebase" |
/// "pull_merge" | "fetch".
#[derive(Serialize, Clone, Debug)]
pub struct Remedy {
    pub id: String,
    /// "safe" | "caution" | "danger" — для цвета кнопки и сортировки внимания.
    pub danger: String,
    /// Рекомендуемое решение для данной ситуации (выделяется в UI).
    pub recommended: bool,
}

/// Распознанная ситуация. `id`: "diverged_rewrite" (локальный коммит —
/// переписанный уже запушенный, безопасен force-with-lease) | "diverged"
/// (на обеих сторонах есть свои коммиты — нужен pull).
#[derive(Serialize, Clone, Debug)]
pub struct Situation {
    pub id: String,
    /// "info" | "warn" | "danger".
    pub severity: String,
    pub ahead: u32,
    pub behind: u32,
    pub remedies: Vec<Remedy>,
}

/// Запускает git и возвращает обрезанный stdout, либо None при ненулевом коде.
fn git_line(repo: &Path, args: &[&str]) -> Option<String> {
    run_git(repo, args).ok().map(|s| s.trim().to_string())
}

/// Диагностирует расхождение локальной ветки с `<remote>/<branch>`.
/// Вызывается после отказа push (non-fast-forward) — к этому моменту фронт
/// уже сделал fetch, поэтому remote-tracking ref актуален.
///
/// Возвращает None, если расхождения нет (behind == 0) или remote-tracking
/// ветка не существует — тогда показывать ассистента не нужно.
pub fn diagnose_sync(repo: &Path, remote: &str, branch: &str) -> Option<Situation> {
    let remote_ref = format!("{}/{}", remote, branch);

    // Remote-tracking ref обязан резолвиться (--quiet → пустой stdout + код 1,
    // если ref'а нет, тогда git_line вернёт None).
    let remote_oid = git_line(repo, &["rev-parse", "--verify", "--quiet", &remote_ref])?;
    if remote_oid.is_empty() {
        return None;
    }
    let head_oid = git_line(repo, &["rev-parse", "HEAD"])?;
    if head_oid == remote_oid {
        return None;
    }

    // ahead\tbehind: коммиты только локально / только на remote.
    let counts = git_line(
        repo,
        &[
            "rev-list",
            "--left-right",
            "--count",
            &format!("{}...{}", head_oid, remote_ref),
        ],
    )?;
    let mut it = counts.split_whitespace();
    let ahead: u32 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    let behind: u32 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);

    // behind == 0 → remote не ушёл вперёд; отказ push не из-за расхождения.
    if behind == 0 {
        return None;
    }

    // Спец-кейс: локальная вершина — переписанный уже запушенный коммит
    // (amend/reword/rebase). Признак — идентичное дерево при разном SHA.
    // Тогда содержимое не теряется и force-with-lease безопасен.
    let head_tree = git_line(repo, &["rev-parse", &format!("{}^{{tree}}", head_oid)]);
    let remote_tree = git_line(repo, &["rev-parse", &format!("{}^{{tree}}", remote_ref)]);
    let rewritten = head_tree.is_some() && head_tree == remote_tree;

    let (id, remedies) = if rewritten {
        // fetch здесь не предлагаем: он уже выполнен до показа ассистента и
        // ничего не решает. Закрыть без действия можно крестиком в баре.
        (
            "diverged_rewrite",
            vec![
                Remedy { id: "push_force_lease".into(), danger: "caution".into(), recommended: true },
            ],
        )
    } else {
        (
            "diverged",
            vec![
                Remedy { id: "pull_rebase".into(), danger: "safe".into(), recommended: true },
                Remedy { id: "pull_merge".into(), danger: "safe".into(), recommended: false },
                Remedy { id: "push_force_lease".into(), danger: "danger".into(), recommended: false },
            ],
        )
    };

    Some(Situation {
        id: id.into(),
        severity: "warn".into(),
        ahead,
        behind,
        remedies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn run(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git").current_dir(dir).args(args).output().unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Свежий репозиторий с одним коммитом на ветке main.
    fn temp_repo() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_diag_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        run(&dir, &["init", "-q", "-b", "main"]);
        run(&dir, &["config", "user.email", "t@t.t"]);
        run(&dir, &["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "hello").unwrap();
        run(&dir, &["add", "."]);
        run(&dir, &["commit", "-qm", "init"]);
        dir
    }

    /// Эмулирует remote-tracking ветку origin/main без реальной сети.
    fn set_remote_ref(dir: &Path, oid: &str) {
        run(dir, &["update-ref", "refs/remotes/origin/main", oid]);
    }

    #[test]
    fn in_sync_returns_none() {
        let dir = temp_repo();
        let head = run(&dir, &["rev-parse", "HEAD"]);
        set_remote_ref(&dir, &head);
        assert!(diagnose_sync(&dir, "origin", "main").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_remote_ref_returns_none() {
        let dir = temp_repo();
        // origin/main не существует — диагностировать нечего.
        assert!(diagnose_sync(&dir, "origin", "main").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn amend_detected_as_rewrite() {
        let dir = temp_repo();
        let c1 = run(&dir, &["rev-parse", "HEAD"]);
        set_remote_ref(&dir, &c1);
        // amend меняет только сообщение → тот же tree, другой SHA.
        run(&dir, &["commit", "--amend", "-qm", "init reworded"]);
        let s = diagnose_sync(&dir, "origin", "main").expect("ожидалась ситуация");
        assert_eq!(s.id, "diverged_rewrite");
        assert!(s.remedies.iter().any(|r| r.id == "push_force_lease" && r.recommended));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn genuine_divergence_recommends_pull() {
        let dir = temp_repo();
        let c1 = run(&dir, &["rev-parse", "HEAD"]);
        // remote ушёл вперёд своим коммитом (другое содержимое).
        fs::write(dir.join("a.txt"), "remote change").unwrap();
        run(&dir, &["commit", "-aqm", "remote work"]);
        let c2 = run(&dir, &["rev-parse", "HEAD"]);
        set_remote_ref(&dir, &c2);
        // Локально возвращаемся на c1 и делаем свой, отличный коммит.
        run(&dir, &["reset", "--hard", "-q", &c1]);
        fs::write(dir.join("b.txt"), "local change").unwrap();
        run(&dir, &["add", "."]);
        run(&dir, &["commit", "-qm", "local work"]);
        let s = diagnose_sync(&dir, "origin", "main").expect("ожидалась ситуация");
        assert_eq!(s.id, "diverged");
        assert!(s.remedies.iter().any(|r| r.id == "pull_rebase" && r.recommended));
        assert!(s.ahead >= 1 && s.behind >= 1);
        fs::remove_dir_all(&dir).ok();
    }
}
