use crate::endpoints::v1::admin::formations::formation_id::delete::endpoint::__path_delete_formation;
use crate::endpoints::v1::admin::formations::formation_id::get::endpoint::__path_get_formation_by_id;
use crate::endpoints::v1::admin::formations::formation_id::get::view::GetFormationByIdResultView;
use crate::endpoints::v1::admin::formations::formation_id::learners::endpoint::__path_get_formation_learners;
use crate::endpoints::v1::admin::formations::formation_id::learners::view::{
    FormationLearner, GetFormationLearnersResultView,
};
use crate::endpoints::v1::admin::formations::formation_id::patch::endpoint::__path_update_formation;
use crate::endpoints::v1::admin::formations::formation_id::register::endpoint::__path_register_user_to_formation;
use crate::endpoints::v1::admin::formations::formation_id::register::view::RegisterUserView;
use crate::endpoints::v1::admin::formations::formation_id::reminders::endpoint::__path_send_reminders;
use crate::endpoints::v1::admin::formations::formation_id::reminders::view::{
    SendRemindersResultView, SendRemindersView,
};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_formation_by_id,
        register_user_to_formation,
        update_formation,
        delete_formation,
        send_reminders,
        get_formation_learners
    ),
    components(schemas(
        GetFormationByIdResultView,
        RegisterUserView,
        SendRemindersView,
        SendRemindersResultView,
        GetFormationLearnersResultView,
        FormationLearner
    ))
)]
pub struct IdDoc;
