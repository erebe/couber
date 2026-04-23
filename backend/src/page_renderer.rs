use crate::database::Video;
use crate::decode_tags;
use maud::{html, Markup, PreEscaped, DOCTYPE};

const CSS: &str = include_str!("../resources/style.css");
const JS: &str = include_str!("../resources/app.js");

fn render_video_card(video: &Video) -> Markup {
    let tags = video
        .tags
        .iter()
        .map(decode_tags)
        .collect::<Vec<_>>()
        .join(", ");
    html! {
        div class="video-card invisible" data-tags=(tags) {
            img data-poster=(video.thumbnail) data-src=(video.url) data-video-name=(video.name) loading="lazy" { }
            div class="video-overlay" {
                span class="video-tags" {}
                button class="video-edit-btn" { "Edit" }
            }
        }
    }
}

fn render_video_grid(videos: &[Video]) -> Markup {
    html! {
        div #videos-container class="videos-grid" {
            @for video in videos {
                (render_video_card(video))
            }
        }
    }
}

pub fn render_page(videos: &[Video]) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Couber" }
                script src="https://unpkg.com/htmx.org@2.0.8" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@tarekraafat/autocomplete.js@10.2.7/dist/css/autoComplete.min.css";
                script src="https://cdn.jsdelivr.net/npm/@tarekraafat/autocomplete.js@10.2.7/dist/autoComplete.min.js" {}
                style { (PreEscaped(CSS)) }
            }
            body {
                header {
                    div class="tag-search" {
                        input #tag-input type="text" placeholder="Filter by tag…" autocomplete="off";
                    }
                    button class="btn btn-icon"
                        onclick="document.getElementById('add-video-dialog').showModal()" {
                        "+"
                    }
                }
                main {
                    (render_video_grid(videos))
                }

                dialog #add-video-dialog
                    onclose="document.getElementById('video-url').value=''; document.getElementById('add-video-status').innerHTML='';" {
                    div class="dialog-header" {
                        h2 { "Add Video" }
                        button class="btn btn-secondary"
                            onclick="document.getElementById('add-video-dialog').close()" {
                            "×"
                        }
                    }
                    form hx-post="/add-video"
                         hx-target="#add-video-status"
                         hx-swap="innerHTML" {
                        div class="dialog-body" {
                            label for="video-url" { "Video URL" }
                            input type="text" name="url" id="video-url"
                                placeholder="https://coub.com/view/... or any video URL"
                                required;
                            div #add-video-status class="status-msg" {}
                        }
                        div class="dialog-footer" {
                            button type="button" class="btn btn-secondary"
                                onclick="document.getElementById('add-video-dialog').close()" {
                                "Close"
                            }
                            button type="submit" class="btn" {
                                "Add Video"
                                span class="htmx-indicator" { "…" }
                            }
                        }
                    }
                }

                dialog #tags-dialog {
                    div class="dialog-header" {
                        h2 { "Edit Tags" }
                        button class="btn btn-secondary"
                            onclick="document.getElementById('tags-dialog').close()" {
                            "×"
                        }
                    }
                    div class="dialog-body" {
                        input type="hidden" id="tags-video-name";
                        div class="tags-section" {
                            label { "Current tags" }
                            div #tags-chips class="chips-container" {}
                        }
                        div class="tags-section" {
                            label { "Add tag" }
                            div class="tag-add-row" {
                                input #tags-new-input type="text" placeholder="Type a tag and press Enter…" autocomplete="off";
                            }
                        }
                        div class="tags-section" {
                            div class="suggest-header" {
                                label { "Suggested tags" }
                                div class="suggest-actions" {
                                    button #select-all-btn type="button" class="btn btn-secondary btn-sm"
                                        onclick="selectAllSuggested()" hidden {
                                        "Select all"
                                    }
                                    button #suggest-btn type="button" class="btn btn-secondary btn-sm"
                                        onclick="suggestTags()" {
                                        "Suggest from image"
                                    }
                                }
                            }
                            div #suggested-chips class="chips-container" {}
                            div #suggest-status class="status-msg" {}
                        }
                        div #tags-status class="status-msg" {}
                    }
                    div class="dialog-footer" {
                        button type="button" class="btn btn-danger"
                            onclick="deleteVideo()" {
                            "Delete video"
                        }
                        div class="footer-right" {
                            button type="button" class="btn btn-secondary"
                                onclick="document.getElementById('tags-dialog').close()" {
                                "Close"
                            }
                            button #save-tags-btn type="button" class="btn"
                                onclick="saveTags()" {
                                "Save Tags"
                            }
                        }
                    }
                    div #delete-status class="status-msg" {}
                }

                script { (PreEscaped(JS)) }
            }
        }
    }
}
