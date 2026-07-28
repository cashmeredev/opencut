#![recursion_limit = "256"]

use ::scene::*;

fn sample_project_json() -> serde_json::Value {
    serde_json::json!({
        "metadata": {
            "id": "proj-1",
            "name": "Demo",
            "duration": 10000,
            "createdAt": "2026-07-28T10:00:00.000Z",
            "updatedAt": "2026-07-28T11:30:00.000Z"
        },
        "scenes": [{
            "id": "scene-1",
            "name": "Main scene",
            "isMain": true,
            "tracks": {
                "overlay": [
                    {
                        "type": "text",
                        "id": "track-text",
                        "name": "Text",
                        "hidden": false,
                        "elements": [{
                            "type": "text",
                            "id": "el-text",
                            "name": "Title",
                            "duration": 2000,
                            "startTime": 0,
                            "trimStart": 0,
                            "trimEnd": 0,
                            "params": { "content": "Hello", "fontSize": 48, "bold": true },
                            "animations": {
                                "opacity": {
                                    "keys": [
                                        { "id": "k1", "time": 0, "value": 0, "segmentToNext": "linear", "tangentMode": "auto" },
                                        { "id": "k2", "time": 500, "value": 1, "leftHandle": { "dt": -100, "dv": -0.1 }, "rightHandle": { "dt": 100, "dv": 0.1 }, "segmentToNext": "bezier", "tangentMode": "aligned" }
                                    ],
                                    "extrapolation": { "before": "hold", "after": "linear" }
                                },
                                "color": {
                                    "r": { "keys": [{ "id": "k3", "time": 0, "value": 0.5, "segmentToNext": "step", "tangentMode": "flat" }] },
                                    "g": { "keys": [{ "id": "k4", "time": 0, "value": 0.2, "segmentToNext": "linear", "tangentMode": "auto" }] }
                                }
                            },
                            "effects": [{ "id": "fx1", "type": "blur", "params": { "radius": 4 }, "enabled": true }]
                        }]
                    }
                ],
                "main": {
                    "type": "video",
                    "id": "track-main",
                    "name": "Main",
                    "muted": false,
                    "hidden": false,
                    "elements": [
                        {
                            "type": "video",
                            "id": "el-video",
                            "name": "Clip",
                            "duration": 5000,
                            "startTime": 0,
                            "trimStart": 100,
                            "trimEnd": 200,
                            "sourceDuration": 6000,
                            "mediaId": "media-1",
                            "isSourceAudioEnabled": true,
                            "retime": { "rate": 1.5, "maintainPitch": true },
                            "params": {},
                            "masks": [
                                { "id": "m1", "type": "rectangle", "params": { "feather": 2, "inverted": false, "strokeColor": "#fff", "strokeWidth": 1, "strokeAlign": "inside", "centerX": 0, "centerY": 0, "width": 100, "height": 50, "rotation": 0, "scale": 1 } },
                                { "id": "m2", "type": "freeform", "params": { "feather": 0, "inverted": true, "strokeColor": "#000", "strokeWidth": 0, "strokeAlign": "center", "path": [{ "id": "p1", "x": 0, "y": 0, "inX": -1, "inY": 0, "outX": 1, "outY": 0 }], "closed": true, "centerX": 10, "centerY": 20, "rotation": 15, "scale": 2 } },
                                { "id": "m3", "type": "cinematic-bars", "params": { "feather": 0, "inverted": false, "strokeColor": "#000", "strokeWidth": 0, "strokeAlign": "outside", "centerX": 0, "centerY": 0, "width": 1920, "height": 200, "rotation": 0, "scale": 1 } }
                            ]
                        },
                        {
                            "type": "image",
                            "id": "el-image",
                            "name": "Still",
                            "duration": 3000,
                            "startTime": 5000,
                            "trimStart": 0,
                            "trimEnd": 0,
                            "mediaId": "media-2",
                            "hidden": false,
                            "params": {}
                        }
                    ]
                },
                "audio": [
                    {
                        "type": "audio",
                        "id": "track-audio",
                        "name": "Audio",
                        "muted": false,
                        "elements": [
                            {
                                "type": "audio",
                                "sourceType": "upload",
                                "id": "el-au1",
                                "name": "Music",
                                "duration": 8000,
                                "startTime": 0,
                                "trimStart": 0,
                                "trimEnd": 0,
                                "mediaId": "media-3",
                                "retime": { "rate": 1 },
                                "params": { "volume": 0.8 }
                            },
                            {
                                "type": "audio",
                                "sourceType": "library",
                                "id": "el-au2",
                                "name": "Sfx",
                                "duration": 1000,
                                "startTime": 2000,
                                "trimStart": 0,
                                "trimEnd": 0,
                                "sourceUrl": "https://example.com/sfx.mp3",
                                "params": {}
                            }
                        ]
                    }
                ]
            },
            "bookmarks": [{ "time": 1500, "note": "cut here", "color": "red", "duration": 250 }],
            "createdAt": "2026-07-28T10:00:00.000Z",
            "updatedAt": "2026-07-28T11:00:00.000Z"
        }],
        "currentSceneId": "scene-1",
        "settings": {
            "fps": { "numerator": 30, "denominator": 1 },
            "canvasSize": { "width": 1920, "height": 1080 },
            "canvasSizeMode": "preset",
            "background": { "type": "blur", "blurIntensity": 8 }
        },
        "version": 7,
        "timelineViewState": { "zoomLevel": 2, "scrollLeft": 120, "playheadTime": 750 }
    })
}

fn diff_path(a: &serde_json::Value, b: &serde_json::Value, path: &str) {
    match (a, b) {
        (serde_json::Value::Object(x), serde_json::Value::Object(y)) => {
            for key in x.keys().chain(y.keys()) {
                if x.get(key) != y.get(key) {
                    diff_path(
                        x.get(key).unwrap_or(&serde_json::Value::Null),
                        y.get(key).unwrap_or(&serde_json::Value::Null),
                        &format!("{path}.{key}"),
                    );
                    return;
                }
            }
        }
        (serde_json::Value::Array(x), serde_json::Value::Array(y)) => {
            for (i, (u, v)) in x.iter().zip(y.iter()).enumerate() {
                if u != v {
                    diff_path(u, v, &format!("{path}[{i}]"));
                    return;
                }
            }
            if x.len() != y.len() {
                panic!("length mismatch at {path}: {} vs {}", x.len(), y.len());
            }
        }
        (serde_json::Value::Number(x), serde_json::Value::Number(y))
            if x.as_f64() == y.as_f64() => {}
        _ => panic!("mismatch at {path}:\n  serialized: {a}\n  original:   {b}"),
    }
}

#[test]
fn project_round_trip_preserves_json() {
    let original = sample_project_json();
    let project: Project = serde_json::from_value(original.clone()).expect("deserialize");
    let serialized = serde_json::to_value(&project).expect("serialize");
    diff_path(&serialized, &original, "$");
}

#[test]
fn element_accessors_reach_base() {
    let original = sample_project_json();
    let project: Project = serde_json::from_value(original).expect("deserialize");
    let main = &project.scenes[0].tracks.main;
    let video = &main.elements()[0];
    assert_eq!(video.base().id, "el-video");
    assert_eq!(video.base().trim_start, time::MediaTime::from_ticks(100));
}
