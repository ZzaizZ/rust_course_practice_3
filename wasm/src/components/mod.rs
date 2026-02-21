//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component  to be used in our app.

mod post_card;
pub use post_card::PostCard;

mod posts_list;
pub use posts_list::PostsList;

mod login_form;
pub use login_form::LoginForm;

mod register_form;
pub use register_form::RegisterForm;

mod authenticated_app;
pub use authenticated_app::AuthenticatedApp;

mod post_form;
pub use post_form::PostForm;

mod post_view;
pub use post_view::PostView;
