use async_trait::async_trait;
use axum::routing::{get, post};
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use sea_orm::DatabaseConnection;

use crate::controllers::{admin, admin_edit, admin_list, auth, config, telegraf};
use crate::middleware::auth::require_auth;

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    fn routes(ctx: &AppContext) -> AppRoutes {
        use loco_rs::controller::Routes;
        
        // 公開ルート
        let public_routes = Routes::new()
            .add("/conf", get(config::show))
            .add("/auth/login", get(auth::login_form))
            .add("/auth/login", post(auth::login));

        // 認証が必要なルート（ミドルウェアを適用）
        let protected_routes = Routes::new()
            .add("/admin", get(admin::index))
            .add("/admin/list", get(admin_list::index))
            .add("/admin/edit", get(admin_edit::edit))
            .add("/admin/edit", post(admin_edit::update))
            .add("/admin/telegraf/restart", post(telegraf::restart))
            .add("/auth/logout", post(auth::logout))
            .layer(axum::middleware::from_fn_with_state(
                ctx.clone(),
                require_auth,
            ));

        AppRoutes::with_default_routes()
            .add_route(public_routes)
            .add_route(protected_routes)
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &loco_rs::bgworker::Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn truncate(_db: &DatabaseConnection) -> Result<()> {
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _base: &std::path::Path) -> Result<()> {
        Ok(())
    }

    async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
        // セッションストアを設定
        use tower_sessions::{MemoryStore, SessionManagerLayer};
        use time::Duration;

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false) // 開発環境ではfalse
            .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::hours(1)));

        Ok(router.layer(session_layer))
    }
}
