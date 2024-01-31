```
--- Requires action
--- tool_call.function.name: "multi_tool_use.parallel"
response (1): Response {
    url: Url {
        scheme: "http",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Domain(
                "homeassistant.local",
            ),
        ),
        port: Some(
            8123,
        ),
        path: "/api/services/multi_tool_use/parallel",
        query: None,
        fragment: None,
    },
    status: 400,
    headers: {
        "content-type": "text/plain; charset=utf-8",
        "referrer-policy": "no-referrer",
        "x-content-type-options": "nosniff",
        "server": "",
        "x-frame-options": "SAMEORIGIN",
        "content-length": "16",
        "date": "Tue, 30 Jan 2024 03:49:15 GMT",
    },
}
	--- Service called: multi_tool_use parallel
	--- Service data: Object {
    "tool_uses": Array [
        Object {
            "parameters": Object {
                "entity_id": Array [
                    String("light.bakrum_shelly_roof"),
                    String("light.bakrum_liten_lampa_lampa"),
                    String("light.bakrum_bord_tak_lampa"),
                    String("light.bakrum_skarm_lampa_lampa"),
                    String("light.all_lights"),
                ],
                "transition": Number(1),
            },
            "recipient_name": String("functions.light.turn_on"),
        },
        Object {
            "parameters": Object {
                "entity_id": Array [
                    String("light.bakrum_shelly_roof"),
                    String("light.bakrum_liten_lampa_lampa"),
                    String("light.bakrum_bord_tak_lampa"),
                    String("light.bakrum_skarm_lampa_lampa"),
                ],
                "transition": Number(10),
            },
            "recipient_name": String("functions.light.turn_off"),
        },
    ],
}
response (2): Err(
    Error {
        context: "Error sending message",
        source: reqwest::Error {
            kind: Decode,
            source: Error("trailing characters", line: 1, column: 4),
        },
    },
)
Failed to call runtime: Error sending message
```
