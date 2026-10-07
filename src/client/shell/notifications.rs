#[cfg(test)]
use super::notification_policy::{COMPLETION_RECHECK_INTERVAL, MAX_VISIBLE_NOTIFICATIONS};
use super::*;
use ratatui::{
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub(super) fn render_mobile_notice_banner(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    body: Option<&str>,
    dot_color: Color,
    offset_for_warning: bool,
    palette: &Palette,
) -> Rect {
    if area.is_empty() {
        return Rect::default();
    }
    let warning_offset = u16::from(offset_for_warning);
    let y = area.y
        + area
            .height
            .saturating_sub(1u16.saturating_add(warning_offset));
    let rect = Rect::new(area.x, y, area.width, 1);
    let background = palette.surface0;
    Clear.render(rect, buffer);
    buffer.set_style(rect, Style::default().bg(background));
    let mut x = rect.x;
    for (text, style) in [
        (" ", Style::default().bg(background)),
        ("●", Style::default().fg(dot_color).bg(background)),
        (" ", Style::default().bg(background)),
        (
            title,
            Style::default()
                .fg(palette.text)
                .bg(background)
                .add_modifier(Modifier::BOLD),
        ),
    ] {
        x = super::render::put_segment(buffer, x, rect.y, rect.right(), text, style);
    }
    if let Some(body) = body.filter(|body| !body.is_empty()) {
        x = super::render::put_segment(
            buffer,
            x,
            rect.y,
            rect.right(),
            " · ",
            Style::default().fg(palette.overlay0).bg(background),
        );
        super::render::put_text(
            buffer,
            x,
            rect.y,
            rect.right().saturating_sub(x),
            body,
            Style::default().fg(palette.overlay0).bg(background),
        );
    }
    rect
}

pub(super) fn render_mobile_notification_banner(
    buffer: &mut Buffer,
    area: Rect,
    notification: &ClientVisibleNotification,
    offset_for_warning: bool,
    palette: &Palette,
) -> Rect {
    let event = &notification.event;
    let title = match event.kind {
        SemanticNotificationKind::NeedsAttention => event
            .title
            .strip_suffix(" needs attention")
            .map(|agent| format!("{agent} waiting"))
            .unwrap_or_else(|| event.title.clone()),
        SemanticNotificationKind::Finished => event
            .title
            .strip_suffix(" finished")
            .map(|agent| format!("{agent} done"))
            .unwrap_or_else(|| event.title.clone()),
        SemanticNotificationKind::UpdateInstalled => "update ready".to_owned(),
        SemanticNotificationKind::Custom => event.title.clone(),
    };
    let dot_color = match event.kind {
        SemanticNotificationKind::NeedsAttention => palette.red,
        SemanticNotificationKind::Finished => palette.blue,
        SemanticNotificationKind::UpdateInstalled | SemanticNotificationKind::Custom => {
            palette.accent
        }
    };
    render_mobile_notice_banner(
        buffer,
        area,
        &title,
        event.body.as_deref(),
        dot_color,
        offset_for_warning,
        palette,
    )
}

fn notification_card_height(body: &str) -> u16 {
    if body.is_empty() {
        3
    } else {
        4
    }
}

pub(super) fn render_notification_card(
    buffer: &mut Buffer,
    area: Rect,
    title: &str,
    body: &str,
    position: crate::config::ToastHerdrPosition,
    top_offset: u16,
    dot_color: Color,
    palette: &Palette,
) -> Rect {
    if area.is_empty() {
        return Rect::default();
    }
    let content_width = unicode_width::UnicodeWidthStr::width(title)
        .max(unicode_width::UnicodeWidthStr::width(body))
        .saturating_add(6);
    let width = u16::try_from(content_width)
        .unwrap_or(u16::MAX)
        .min(area.width);
    let height = notification_card_height(body).min(area.height);
    let x = match position {
        crate::config::ToastHerdrPosition::TopLeft
        | crate::config::ToastHerdrPosition::BottomLeft => area.x,
        crate::config::ToastHerdrPosition::TopRight
        | crate::config::ToastHerdrPosition::BottomRight => area.right().saturating_sub(width),
    };
    let max_y = area.bottom().saturating_sub(height).max(area.y);
    let y = match position {
        crate::config::ToastHerdrPosition::TopLeft
        | crate::config::ToastHerdrPosition::TopRight => area.y.saturating_add(top_offset),
        crate::config::ToastHerdrPosition::BottomLeft
        | crate::config::ToastHerdrPosition::BottomRight => area
            .bottom()
            .saturating_sub(height.saturating_add(top_offset)),
    }
    .clamp(area.y, max_y);
    let rect = Rect::new(x, y, width, height);
    Clear.render(rect, buffer);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(palette.overlay0))
        .style(Style::default().bg(palette.panel_bg));
    let inner = block.inner(rect);
    block.render(rect, buffer);
    Paragraph::new(Line::from(vec![
        Span::styled("●", Style::default().fg(dot_color)),
        Span::raw(" "),
        Span::styled(
            title,
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .render(Rect::new(inner.x, inner.y, inner.width, 1), buffer);
    if !body.is_empty() && inner.height > 1 {
        Paragraph::new(Line::from(Span::styled(
            body,
            Style::default().fg(palette.overlay0),
        )))
        .render(
            Rect::new(
                inner.x.saturating_add(2),
                inner.y + 1,
                inner.width.saturating_sub(2),
                1,
            ),
            buffer,
        );
    }
    rect
}

pub(super) fn render_visible_notification(
    buffer: &mut Buffer,
    area: Rect,
    notification: &ClientVisibleNotification,
    default_position: crate::config::ToastHerdrPosition,
    top_offset: u16,
    palette: &Palette,
) -> Rect {
    let event = &notification.event;
    let dot_color = match event.kind {
        SemanticNotificationKind::NeedsAttention => palette.red,
        SemanticNotificationKind::Finished => palette.blue,
        SemanticNotificationKind::UpdateInstalled | SemanticNotificationKind::Custom => {
            palette.accent
        }
    };
    render_notification_card(
        buffer,
        area,
        &event.title,
        event.body.as_deref().unwrap_or_default(),
        event.position.unwrap_or(default_position),
        top_offset,
        dot_color,
        palette,
    )
}

/// Draws the toasts stacked from the edge their position names, the newest
/// nearest the edge. Past the first at each position, a toast that no longer
/// fits in the area is left out, and so are the older ones behind it. Returns
/// each card drawn with its index in `notifications`.
pub(super) fn render_visible_notifications(
    buffer: &mut Buffer,
    area: Rect,
    notifications: &[ClientVisibleNotification],
    default_position: crate::config::ToastHerdrPosition,
    top_offset: u16,
    palette: &Palette,
) -> Vec<(Rect, usize)> {
    let mut offsets = [Some(top_offset); 4];
    let mut rects = Vec::new();
    for (index, notification) in notifications.iter().enumerate().rev() {
        let position = notification.event.position.unwrap_or(default_position);
        let slot = &mut offsets[match position {
            crate::config::ToastHerdrPosition::TopLeft => 0,
            crate::config::ToastHerdrPosition::TopRight => 1,
            crate::config::ToastHerdrPosition::BottomLeft => 2,
            crate::config::ToastHerdrPosition::BottomRight => 3,
        }];
        let Some(offset) = *slot else {
            continue;
        };
        let height =
            notification_card_height(notification.event.body.as_deref().unwrap_or_default());
        if offset > top_offset && offset.saturating_add(height) > area.height {
            *slot = None;
            continue;
        }
        let rect = render_visible_notification(
            buffer,
            area,
            notification,
            default_position,
            offset,
            palette,
        );
        *slot = Some(offset.saturating_add(rect.height));
        rects.push((rect, index));
    }
    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notification() -> ClientVisibleNotification {
        ClientVisibleNotification {
            endpoint_id: ClientEndpointId::Local,
            event: SemanticNotification {
                kind: SemanticNotificationKind::Custom,
                title: "notice".into(),
                body: None,
                sound: None,
                agent: None,
                workspace_id: None,
                tab_id: None,
                pane_id: None,
                position: None,
            },
            deadline: None,
        }
    }

    #[test]
    fn mobile_notification_is_a_bottom_banner_with_released_title() {
        let palette = crate::app::client_palette_from_config(&Config::default());
        let mut notification = notification();
        notification.event.kind = SemanticNotificationKind::NeedsAttention;
        notification.event.title = "pi needs attention".into();
        notification.event.body = Some("workspace · tab 1".into());
        let area = Rect::new(0, 0, 44, 20);
        let mut buffer = Buffer::empty(area);
        for cell in &mut buffer.content {
            cell.set_symbol("X");
        }
        let rect =
            render_mobile_notification_banner(&mut buffer, area, &notification, true, &palette);
        assert_eq!(rect, Rect::new(0, 18, 44, 1));
        let text = buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("pi waiting"));
        assert!(text.contains("workspace · tab 1"));
        assert!(buffer.content[18 * 44..19 * 44]
            .iter()
            .all(|cell| cell.symbol() != "X"));
    }

    #[test]
    fn unavailable_notification_target_stays_visible_and_reports_the_machine() {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        state.visible_notifications = vec![ClientVisibleNotification {
            endpoint_id: ClientEndpointId::Local,
            event: SemanticNotification {
                kind: SemanticNotificationKind::NeedsAttention,
                title: "agent needs attention".into(),
                body: None,
                sound: None,
                agent: Some("agent".into()),
                workspace_id: Some("workspace".into()),
                tab_id: Some("tab".into()),
                pane_id: Some("pane".into()),
                position: None,
            },
            deadline: Some(std::time::Instant::now() + std::time::Duration::from_secs(5)),
        }];
        let mut outcome = ClientShellInput::default();

        state.focus_visible_notification(&mut outcome);

        assert!(!state.visible_notifications.is_empty());
        assert!(state
            .visible_endpoint_notice
            .as_ref()
            .is_some_and(|notice| notice.body.contains("Local is unavailable")));
        assert!(outcome.repaint);
    }

    #[test]
    fn immediate_finished_evidence_for_acknowledged_idle_never_toasts_or_sounds() {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.toast_delivery = crate::config::ToastDelivery::Herdr;
        config.toast_delay_seconds = 0;
        let mut state = ClientShellState::new(config);
        let mut snapshot = super::super::tests::snapshot();
        snapshot.agents.push(crate::protocol::ClientShellAgent {
            pane_id: "pane_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("agent".into()),
            display_agent: None,
            agent: Some("agent".into()),
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: crate::api::schema::AgentStatus::Idle,
            state_change_seq: 2,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: true,
        });
        state.set_snapshot(Box::new(snapshot));

        let (effects, repaint) = state.receive_notification(
            &ClientEndpointId::Local,
            SemanticNotification {
                kind: SemanticNotificationKind::Finished,
                title: "agent finished".into(),
                body: None,
                sound: Some(SemanticNotificationSound::Done),
                agent: Some("agent".into()),
                workspace_id: Some("ws_1".into()),
                tab_id: Some("tab_1".into()),
                pane_id: Some("pane_1".into()),
                position: None,
            },
            std::time::Instant::now(),
        );

        assert!(effects.is_empty());
        assert!(!repaint);
        assert!(state.visible_notifications.is_empty());
    }

    #[test]
    fn finished_hint_waits_for_the_projected_completion_snapshot() {
        let mut config = ClientShellConfig::from_config(&Config::default());
        config.toast_delivery = crate::config::ToastDelivery::Terminal;
        config.toast_delay_seconds = 0;
        let mut state = ClientShellState::new(config);
        let mut snapshot = super::super::tests::snapshot();
        snapshot.agents.push(crate::protocol::ClientShellAgent {
            pane_id: "pane_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            name: Some("agent".into()),
            display_agent: None,
            agent: Some("agent".into()),
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            agent_status: crate::api::schema::AgentStatus::Working,
            state_change_seq: 1,
            state_labels: Vec::new(),
            tokens: Vec::new(),
            focused: false,
        });
        state.set_snapshot(Box::new(snapshot.clone()));
        let now = std::time::Instant::now();
        let event = SemanticNotification {
            kind: SemanticNotificationKind::Finished,
            title: "agent finished".into(),
            body: None,
            sound: None,
            agent: Some("agent".into()),
            workspace_id: Some("ws_1".into()),
            tab_id: Some("background-tab".into()),
            pane_id: Some("pane_1".into()),
            position: None,
        };

        let (effects, _) = state.receive_notification(&ClientEndpointId::Local, event, now);
        assert!(effects.is_empty());
        assert_eq!(state.pending_notifications.len(), 1);

        snapshot.revision = snapshot.revision.saturating_add(1);
        snapshot.agents[0].agent_status = crate::api::schema::AgentStatus::Idle;
        snapshot.agents[0].state_change_seq = 2;
        state.set_snapshot(Box::new(snapshot));
        let (effects, _) = state.tick_notifications(now + COMPLETION_RECHECK_INTERVAL);
        assert!(matches!(
            effects.as_slice(),
            [ClientShellNotificationEffect::Terminal { title, .. }] if title == "agent finished"
        ));
        assert!(state.pending_notifications.is_empty());
    }

    #[test]
    fn retiring_one_endpoint_preserves_other_endpoint_notifications() {
        let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
        let mut local = notification();
        local.event.title = "local".into();
        let remote_id = ClientEndpointId::Ssh(
            crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef").unwrap(),
        );
        let mut remote = notification();
        remote.endpoint_id = remote_id.clone();
        remote.event.title = "remote".into();
        let mut later_local = notification();
        later_local.event.title = "later local".into();
        state.visible_notifications = vec![local, remote, later_local];

        state.retire_endpoint_notifications(&ClientEndpointId::Local);

        assert_eq!(
            state
                .visible_notifications
                .iter()
                .map(|notification| (&notification.endpoint_id, notification.event.title.as_str()))
                .collect::<Vec<_>>(),
            vec![(&remote_id, "remote")]
        );
    }

    fn visible_titles(state: &ClientShellState) -> Vec<&str> {
        state
            .visible_notifications
            .iter()
            .map(|notification| notification.event.title.as_str())
            .collect()
    }

    #[test]
    fn concurrent_notifications_are_shown_together_and_expire_on_their_own() {
        let mut state = herdr_toast_state(None);
        let now = std::time::Instant::now();
        let remote_id = ClientEndpointId::Ssh(
            crate::client::endpoint::ProfileId::parse("0123456789abcdef0123456789abcdef").unwrap(),
        );
        state.receive_notification(
            &ClientEndpointId::Local,
            custom_notification("first", Some("pane_1")),
            now,
        );
        state.receive_notification(
            &remote_id,
            custom_notification("second", Some("pane_1")),
            now + std::time::Duration::from_secs(2),
        );

        assert_eq!(visible_titles(&state), vec!["first", "second"]);

        let (_, repaint) = state.tick_notifications(now + std::time::Duration::from_secs(5));
        assert!(repaint);
        assert_eq!(visible_titles(&state), vec!["second"]);

        state.tick_notifications(now + std::time::Duration::from_secs(7));
        assert!(state.visible_notifications.is_empty());
    }

    #[test]
    fn a_new_notification_from_a_pane_replaces_that_panes_toast() {
        let mut state = herdr_toast_state(None);
        let now = std::time::Instant::now();
        for (title, pane_id) in [
            ("one", "pane_1"),
            ("two", "pane_2"),
            ("one again", "pane_1"),
        ] {
            state.receive_notification(
                &ClientEndpointId::Local,
                custom_notification(title, Some(pane_id)),
                now,
            );
        }

        assert_eq!(visible_titles(&state), vec!["two", "one again"]);
    }

    #[test]
    fn notifications_past_the_limit_close_the_oldest() {
        let mut state = herdr_toast_state(None);
        let now = std::time::Instant::now();
        let titles = (0..=MAX_VISIBLE_NOTIFICATIONS)
            .map(|index| format!("toast {index}"))
            .collect::<Vec<_>>();
        for title in &titles {
            state.receive_notification(
                &ClientEndpointId::Local,
                custom_notification(title, Some(title)),
                now,
            );
        }

        assert_eq!(
            visible_titles(&state),
            titles[1..].iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    #[test]
    fn stacked_notifications_put_the_newest_at_the_edge_and_drop_what_does_not_fit() {
        let palette = crate::app::client_palette_from_config(&Config::default());
        let notifications = ["oldest", "middle", "newest"]
            .into_iter()
            .map(|title| {
                let mut notification = notification();
                notification.event.title = title.into();
                notification
            })
            .collect::<Vec<_>>();
        for (position, edge_y) in [
            (crate::config::ToastHerdrPosition::BottomRight, 17),
            (crate::config::ToastHerdrPosition::TopLeft, 0),
        ] {
            let area = Rect::new(0, 0, 40, 20);
            let mut buffer = Buffer::empty(area);
            let rects = render_visible_notifications(
                &mut buffer,
                area,
                &notifications,
                position,
                0,
                &palette,
            );

            assert_eq!(
                rects.iter().map(|(_, index)| *index).collect::<Vec<_>>(),
                vec![2, 1, 0]
            );
            assert_eq!(rects[0].0.y, edge_y);
            for pair in rects.windows(2) {
                let (nearer, farther) = (pair[0].0, pair[1].0);
                assert!(nearer.bottom() <= farther.y || farther.bottom() <= nearer.y);
            }

            // Two 3-row cards fit in 7 rows; the oldest is left out.
            let short = Rect::new(0, 0, 40, 7);
            let mut buffer = Buffer::empty(short);
            let rects = render_visible_notifications(
                &mut buffer,
                short,
                &notifications,
                position,
                0,
                &palette,
            );
            assert_eq!(
                rects.iter().map(|(_, index)| *index).collect::<Vec<_>>(),
                vec![2, 1]
            );
        }
    }

    fn custom_notification(title: &str, pane_id: Option<&str>) -> SemanticNotification {
        SemanticNotification {
            kind: SemanticNotificationKind::Custom,
            title: title.into(),
            body: None,
            sound: None,
            agent: None,
            workspace_id: None,
            tab_id: None,
            pane_id: pane_id.map(Into::into),
            position: None,
        }
    }

    fn herdr_toast_state(duration_seconds: Option<u64>) -> ClientShellState {
        let mut config = Config::default();
        config.ui.toast.delivery = crate::config::ToastDelivery::Herdr;
        config.ui.toast.duration_seconds = duration_seconds;
        ClientShellState::new(ClientShellConfig::from_config(&config))
    }

    #[test]
    fn unset_toast_duration_keeps_each_kind_its_own_duration() {
        let mut state = herdr_toast_state(None);
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            custom_notification("custom", None),
            now,
        );

        state.tick_notifications(now + std::time::Duration::from_millis(4999));
        assert!(!state.visible_notifications.is_empty());
        state.tick_notifications(now + std::time::Duration::from_secs(5));
        assert!(state.visible_notifications.is_empty());
    }

    #[test]
    fn configured_toast_duration_applies_to_every_kind() {
        let mut state = herdr_toast_state(Some(2));
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            custom_notification("custom", None),
            now,
        );

        state.tick_notifications(now + std::time::Duration::from_millis(1999));
        assert!(!state.visible_notifications.is_empty());
        state.tick_notifications(now + std::time::Duration::from_secs(2));
        assert!(state.visible_notifications.is_empty());
    }

    #[test]
    fn zero_toast_duration_keeps_the_toast_until_it_is_closed() {
        let mut state = herdr_toast_state(Some(0));
        let now = std::time::Instant::now();
        state.receive_notification(
            &ClientEndpointId::Local,
            custom_notification("custom", None),
            now,
        );

        state.tick_notifications(now + std::time::Duration::from_secs(24 * 60 * 60));
        assert!(!state.visible_notifications.is_empty());

        let mut outcome = ClientShellInput::default();
        state.focus_visible_notification(&mut outcome);
        assert!(state.visible_notifications.is_empty());
    }

    #[test]
    fn notification_rect_stays_inside_short_nonzero_area() {
        let palette = crate::app::client_palette_from_config(&Config::default());
        for height in [1, 2] {
            let area = Rect::new(3, 4, 8, height);
            for position in [
                crate::config::ToastHerdrPosition::TopRight,
                crate::config::ToastHerdrPosition::BottomRight,
            ] {
                let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
                let rect = render_visible_notification(
                    &mut buffer,
                    area,
                    &notification(),
                    position,
                    1,
                    &palette,
                );
                assert!(rect.y >= area.y);
                assert!(rect.bottom() <= area.bottom());
            }
        }
    }
}
