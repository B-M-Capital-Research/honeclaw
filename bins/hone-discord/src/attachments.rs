use hone_channels::attachments::{
    RawAttachment, ReceivedAttachment, build_user_input, build_user_input_with_label,
};
use serenity::all::Message;

pub(crate) async fn collect_raw_attachments(msg: &Message) -> Vec<RawAttachment> {
    if msg.attachments.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(msg.attachments.len());
    for att in &msg.attachments {
        let mut received = RawAttachment {
            filename: att.filename.clone(),
            content_type: att.content_type.clone(),
            size: Some(att.size),
            url: att.url.clone(),
            local_path: None,
            data: None,
            error: None,
        };

        match att.download().await {
            Ok(bytes) => {
                received.data = Some(bytes);
            }
            Err(e) => {
                received.error = Some(format!("下载失败: {e}"));
            }
        }

        out.push(received);
    }

    out
}

pub(crate) fn build_dm_user_input(content: &str, attachments: &[ReceivedAttachment]) -> String {
    build_user_input(content, attachments)
}

pub(crate) fn build_group_user_input(content: &str, attachments: &[ReceivedAttachment]) -> String {
    build_user_input_with_label(content, attachments, "附带了文件：")
}
