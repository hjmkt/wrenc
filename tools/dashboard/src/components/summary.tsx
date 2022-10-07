import React from "react";
import { Scatter, Bar } from "react-chartjs-2";
import { Chart, registerables } from "chart.js";
import {
    Spinner,
    Card,
    Container,
    Row,
    Col,
    Table,
    Nav,
} from "react-bootstrap";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFileVideo } from "@fortawesome/free-solid-svg-icons";
import SummaryData from "../../../evaluation/summary.json";
import VideoData from "../../../evaluation/videos.json";
Chart.register(...registerables);
Chart.defaults.color = "rgba(255, 255, 255, 1)";

const Summary = () => {
    const [summary, setSummary] = React.useState({});
    const plugin = {
        id: "custom_canvas_background_color",
        beforeDraw: (chart) => {
            const { ctx } = chart;
            ctx.save();
            ctx.globalCompositeOperation = "destination-over";
            ctx.fillStyle = "rgba(0, 0, 0, 0)";
            ctx.fillRect(0, 0, chart.width, chart.height);
            ctx.restore();
        },
    };
    React.useEffect(() => {
        let results = {};
        for (let video of Object.keys(VideoData)) {
            results[video] = {};
        }
        for (const preset_result of SummaryData["results"]) {
            let preset = preset_result["preset"];
            for (let parameter_result of preset_result["results"]) {
                //let parameters = parameter_result["parameters"];
                let tag = parameter_result["tag"];
                for (let video_result of parameter_result["results"]) {
                    let video = video_result["video"];
                    results[video][tag] = video_result;
                }
            }
        }
        setSummary(results);
    }, []);
    const palette = [
        "rgba(64, 128, 224, 1)",
        "rgba(224, 64, 128, 1)",
        "rgba(128, 224, 64, 1)",
        "rgba(64, 224, 128, 1)",
        "rgba(128, 64, 224, 1)",
        "rgba(224, 128, 64, 1)",
    ];
    let date = SummaryData["date"];
    let commitId = SummaryData["commit_id"];
    let commitMessage = SummaryData["commit_message"];
    if (Object.keys(summary).length == 0)
        return (
            <div
                style={{
                    display: "table",
                    width: "100%",
                    height: "100%",
                    top: 0,
                    left: 0,
                    position: "fixed",
                }}
            >
                <div
                    style={{
                        display: "table-cell",
                        textAlign: "center",
                        verticalAlign: "middle",
                    }}
                >
                    <Spinner
                        animation="border"
                        role="status"
                        variant="info"
                        style={{ width: "5rem", height: "5rem" }}
                    >
                        <span className="sr-only">Loading...</span>
                    </Spinner>
                </div>
            </div>
        );
    return (
        <div className="pt-1" style={{ backgroundColor: "rgba(0, 0, 0, 0.8)" }}>
            <Card
                className="m-2 px-1"
                style={{
                    borderWidth: 3,
                    borderColor: "rgba(0, 0, 0, 0.7)",
                    backgroundColor: "rgba(0, 0, 0, 0.5)",
                }}
            >
                <Card.Body
                    className="py-2"
                    style={{
                        color: "rgba(255, 255, 255, 0.85)",
                        fontSize: "1.2vw",
                        fontWeight: "bold",
                    }}
                >
                    Last updated at {date} with commit#{commitId} [
                    {commitMessage}]
                </Card.Body>
            </Card>
            {Object.keys(summary).map((video_name) => (
                <Card
                    className="m-2 px-1"
                    style={{
                        borderWidth: 3,
                        borderColor: "rgba(0, 0, 0, 0.7)",
                        backgroundColor: "rgba(0, 0, 0, 0.5)",
                    }}
                >
                    <Card.Body className="py-2">
                        <div style={{ width: "100%" }}>
                            <Row>
                                <Col sm={3} className="m-0 p-1">
                                    <Card
                                        style={{
                                            width: "100%",
                                            borderWidth: 2,
                                            float: "left",
                                            backgroundColor:
                                                "rgba(0, 0, 0, 0.2)",
                                        }}
                                    >
                                        <Card.Body className="text-center">
                                            <Card.Title>
                                                <div
                                                    className="py-2"
                                                    style={{
                                                        color: "rgba(255, 255, 255, 0.85)",
                                                        fontSize: "1vw",
                                                        fontWeight: "bold",
                                                        border: "solid 0.15em",
                                                        borderRadius: "0.3em",
                                                    }}
                                                >
                                                    <FontAwesomeIcon
                                                        icon={faFileVideo}
                                                        className="mr-2"
                                                    />
                                                    {`${
                                                        video_name.split(".")[0]
                                                    }`}
                                                </div>
                                            </Card.Title>
                                            <video
                                                src={video_name}
                                                controls
                                                style={{ width: "100%" }}
                                            ></video>
                                            <Table
                                                hover
                                                style={{ color: "white" }}
                                            >
                                                <tbody>
                                                    <tr>
                                                        <td>Width</td>
                                                        <td className="text-center">
                                                            {
                                                                VideoData[
                                                                    video_name
                                                                ]["width"]
                                                            }{" "}
                                                            px
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td>Height</td>
                                                        <td className="text-center">
                                                            {
                                                                VideoData[
                                                                    video_name
                                                                ]["height"]
                                                            }{" "}
                                                            px
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td>Frame rate</td>
                                                        <td className="text-center">
                                                            {
                                                                VideoData[
                                                                    video_name
                                                                ]["frame_rate"]
                                                            }{" "}
                                                            fps
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td>
                                                            Number of frames
                                                        </td>
                                                        <td className="text-center">
                                                            {
                                                                VideoData[
                                                                    video_name
                                                                ]["frames"]
                                                            }
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td>Chroma Format</td>
                                                        <td className="text-center">
                                                            YUV420
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td>Source</td>
                                                        <td className="text-center">
                                                            <a
                                                                href={
                                                                    VideoData[
                                                                        video_name
                                                                    ]["source"]
                                                                }
                                                            >
                                                                {VideoData[
                                                                    video_name
                                                                ]["source"]
                                                                    .split(
                                                                        "/"
                                                                    )[2]
                                                                    .split(".")
                                                                    .slice(-2)
                                                                    .join(".")}
                                                            </a>
                                                        </td>
                                                    </tr>
                                                </tbody>
                                            </Table>
                                        </Card.Body>
                                    </Card>
                                </Col>
                                <Col sm={9} className="m-0 p-1">
                                    <Card
                                        style={{
                                            width: "100%",
                                            height: "100%",
                                            borderWidth: 2,
                                            float: "left",
                                            backgroundColor:
                                                "rgba(0, 0, 0, 0.2)",
                                        }}
                                    >
                                        <Card.Body className="text-center">
                                            <Container className="p-0">
                                                <Row sm={6}>
                                                    <Col
                                                        sm={6}
                                                        className="m-0 p-1"
                                                    >
                                                        <Scatter
                                                            data={{
                                                                datasets:
                                                                    Object.keys(
                                                                        summary[
                                                                            video_name
                                                                        ]
                                                                    ).map(
                                                                        (
                                                                            tag,
                                                                            idx
                                                                        ) => ({
                                                                            label: tag,
                                                                            data: (() => {
                                                                                let xData =
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].map(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            (r[
                                                                                                "bytes"
                                                                                            ] *
                                                                                                30.0) /
                                                                                            30.0 /
                                                                                            1024.0
                                                                                    );
                                                                                let yData =
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].map(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            r[
                                                                                                "metrics"
                                                                                            ][
                                                                                                "PSNR"
                                                                                            ][
                                                                                                "summary"
                                                                                            ][
                                                                                                "Avg"
                                                                                            ]
                                                                                    );
                                                                                let data =
                                                                                    [];
                                                                                for (
                                                                                    let i = 0;
                                                                                    i <
                                                                                    xData.length;
                                                                                    i++
                                                                                ) {
                                                                                    data.push(
                                                                                        {
                                                                                            x: xData[
                                                                                                i
                                                                                            ],
                                                                                            y: yData[
                                                                                                i
                                                                                            ],
                                                                                        }
                                                                                    );
                                                                                }
                                                                                data.sort(
                                                                                    (
                                                                                        a,
                                                                                        b
                                                                                    ) =>
                                                                                        a[
                                                                                            "x"
                                                                                        ] <
                                                                                        b[
                                                                                            "x"
                                                                                        ]
                                                                                            ? -1
                                                                                            : 1
                                                                                );
                                                                                return data;
                                                                            })(),
                                                                            fill: false,
                                                                            showLine:
                                                                                true,
                                                                            tension: 0.2,
                                                                            borderColor:
                                                                                palette[
                                                                                    idx
                                                                                ],
                                                                            backgroundColor:
                                                                                palette[
                                                                                    idx
                                                                                ],
                                                                            borderWidth: 1,
                                                                            pointRadius: 2,
                                                                        })
                                                                    ),
                                                            }}
                                                            plugins={[plugin]}
                                                            options={{
                                                                responsive:
                                                                    true,
                                                                scales: {
                                                                    x: {
                                                                        title: {
                                                                            text: "Bitrate [kbps]",
                                                                            display:
                                                                                true,
                                                                        },
                                                                        grid: {
                                                                            //color: "rgba(255, 255, 255, 0.2)",
                                                                        },
                                                                    },
                                                                    y: {
                                                                        title: {
                                                                            text: "PSNR",
                                                                            display:
                                                                                true,
                                                                        },
                                                                        grid: {
                                                                            //color: "rgba(255, 255, 255, 0.2)",
                                                                        },
                                                                    },
                                                                },
                                                            }}
                                                        />
                                                    </Col>
                                                    <Col
                                                        sm={6}
                                                        className="m-0 p-1"
                                                    >
                                                        <Bar
                                                            data={{
                                                                labels: Object.keys(
                                                                    summary[
                                                                        video_name
                                                                    ]
                                                                ),
                                                                datasets: [
                                                                    {
                                                                        //label: "Encoding Speed [x real-time]",
                                                                        data: Object.keys(
                                                                            summary[
                                                                                video_name
                                                                            ]
                                                                        ).map(
                                                                            (
                                                                                tag,
                                                                                idx
                                                                            ) => {
                                                                                let speeds =
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].map(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            (r[
                                                                                                "duration"
                                                                                            ] *
                                                                                                30.0) /
                                                                                            30.0
                                                                                    );
                                                                                let avgSpeed = 0;
                                                                                for (
                                                                                    let i = 0;
                                                                                    i <
                                                                                    speeds.length;
                                                                                    i++
                                                                                ) {
                                                                                    avgSpeed +=
                                                                                        speeds[
                                                                                            i
                                                                                        ];
                                                                                }
                                                                                avgSpeed /=
                                                                                    speeds.length;
                                                                                return avgSpeed;
                                                                            }
                                                                        ),
                                                                        //fill: true,
                                                                        //showLine: true,
                                                                        //tension: 0.2,
                                                                        borderColor:
                                                                            palette.slice(
                                                                                0,
                                                                                Object.keys(
                                                                                    summary[
                                                                                        video_name
                                                                                    ]
                                                                                )
                                                                                    .length
                                                                            ),
                                                                        backgroundColor:
                                                                            palette.slice(
                                                                                0,
                                                                                Object.keys(
                                                                                    summary[
                                                                                        video_name
                                                                                    ]
                                                                                )
                                                                                    .length
                                                                            ),
                                                                        borderWidth: 1,
                                                                    },
                                                                ],
                                                            }}
                                                            plugins={[plugin]}
                                                            options={{
                                                                indexAxis: "y",
                                                                plugins: {
                                                                    legend: {
                                                                        display:
                                                                            false,
                                                                    },
                                                                },
                                                                scales: {
                                                                    x: {
                                                                        title: {
                                                                            text: "Encoding Speed [x real-time]",
                                                                            display:
                                                                                true,
                                                                        },
                                                                    },
                                                                },
                                                            }}
                                                        />
                                                    </Col>
                                                </Row>
                                                <Row sm={6}>
                                                    <Col
                                                        sm={6}
                                                        className="m-0 p-1"
                                                    >
                                                        <Scatter
                                                            data={{
                                                                datasets:
                                                                    Object.keys(
                                                                        summary[
                                                                            video_name
                                                                        ]
                                                                    ).map(
                                                                        (
                                                                            tag,
                                                                            idx
                                                                        ) => ({
                                                                            label: tag,
                                                                            data: (() => {
                                                                                let xData =
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].map(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            (r[
                                                                                                "bytes"
                                                                                            ] *
                                                                                                30.0) /
                                                                                            30.0 /
                                                                                            1024.0
                                                                                    );
                                                                                let yData =
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].map(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            r[
                                                                                                "metrics"
                                                                                            ][
                                                                                                "SSIM"
                                                                                            ][
                                                                                                "summary"
                                                                                            ][
                                                                                                "Avg"
                                                                                            ]
                                                                                    );
                                                                                let data =
                                                                                    [];
                                                                                for (
                                                                                    let i = 0;
                                                                                    i <
                                                                                    xData.length;
                                                                                    i++
                                                                                ) {
                                                                                    data.push(
                                                                                        {
                                                                                            x: xData[
                                                                                                i
                                                                                            ],
                                                                                            y: yData[
                                                                                                i
                                                                                            ],
                                                                                        }
                                                                                    );
                                                                                }
                                                                                data.sort(
                                                                                    (
                                                                                        a,
                                                                                        b
                                                                                    ) =>
                                                                                        a[
                                                                                            "x"
                                                                                        ] <
                                                                                        b[
                                                                                            "x"
                                                                                        ]
                                                                                            ? -1
                                                                                            : 1
                                                                                );
                                                                                return data;
                                                                            })(),
                                                                            fill: false,
                                                                            showLine:
                                                                                true,
                                                                            tension: 0.2,
                                                                            borderColor:
                                                                                palette[
                                                                                    idx
                                                                                ],
                                                                            backgroundColor:
                                                                                palette[
                                                                                    idx
                                                                                ],
                                                                            borderWidth: 1,
                                                                            pointRadius: 2,
                                                                        })
                                                                    ),
                                                            }}
                                                            plugins={[plugin]}
                                                            options={{
                                                                responsive:
                                                                    true,
                                                                scales: {
                                                                    x: {
                                                                        title: {
                                                                            text: "Bitrate [kbps]",
                                                                            display:
                                                                                true,
                                                                        },
                                                                        grid: {
                                                                            //color: "rgba(255, 255, 255, 0.2)",
                                                                        },
                                                                    },
                                                                    y: {
                                                                        title: {
                                                                            text: "SSIM",
                                                                            display:
                                                                                true,
                                                                        },
                                                                        grid: {
                                                                            //color: "rgba(255, 255, 255, 0.2)",
                                                                        },
                                                                    },
                                                                },
                                                            }}
                                                        />
                                                    </Col>
                                                    <Col
                                                        sm={6}
                                                        className="m-0 p-1"
                                                    >
                                                        <Bar
                                                            data={{
                                                                labels: Array.from(
                                                                    {
                                                                        length: VideoData[
                                                                            video_name
                                                                        ][
                                                                            "frames"
                                                                        ],
                                                                    },
                                                                    (_, i) =>
                                                                        i + 1
                                                                ),
                                                                datasets:
                                                                    Object.keys(
                                                                        summary[
                                                                            video_name
                                                                        ]
                                                                    )
                                                                        .map(
                                                                            (
                                                                                tag
                                                                            ) => {
                                                                                return [
                                                                                    tag,
                                                                                    summary[
                                                                                        video_name
                                                                                    ][
                                                                                        tag
                                                                                    ][
                                                                                        "results"
                                                                                    ].filter(
                                                                                        (
                                                                                            r
                                                                                        ) =>
                                                                                            r[
                                                                                                "qp"
                                                                                            ] ==
                                                                                            26
                                                                                    )[0],
                                                                                ];
                                                                            }
                                                                        )
                                                                        .map(
                                                                            (
                                                                                r,
                                                                                idx
                                                                            ) => ({
                                                                                label: r[0],
                                                                                data: (() => {
                                                                                    let psnrs =
                                                                                        r[1][
                                                                                            "metrics"
                                                                                        ][
                                                                                            "PSNR"
                                                                                        ][
                                                                                            "per_frame"
                                                                                        ];
                                                                                    let data =
                                                                                        [];
                                                                                    for (
                                                                                        let i = 0;
                                                                                        i <
                                                                                        psnrs.length;
                                                                                        i++
                                                                                    ) {
                                                                                        data.push(
                                                                                            psnrs[
                                                                                                i
                                                                                            ][
                                                                                                "Avg"
                                                                                            ]
                                                                                        );
                                                                                    }
                                                                                    return data;
                                                                                })(),
                                                                                borderColor:
                                                                                    palette[
                                                                                        idx
                                                                                    ],
                                                                                backgroundColor:
                                                                                    palette[
                                                                                        idx
                                                                                    ],
                                                                            })
                                                                        ),
                                                            }}
                                                            plugins={[plugin]}
                                                            options={{
                                                                scales: {
                                                                    x: {
                                                                        title: {
                                                                            text: "Frame",
                                                                            display:
                                                                                true,
                                                                        },
                                                                    },
                                                                    y: {
                                                                        title: {
                                                                            text: "PSNR per frame @qp=26 [dB]",
                                                                            display:
                                                                                true,
                                                                        },
                                                                    },
                                                                },
                                                            }}
                                                        />
                                                    </Col>
                                                </Row>
                                            </Container>
                                        </Card.Body>
                                    </Card>
                                </Col>
                            </Row>
                        </div>
                    </Card.Body>
                </Card>
            ))}
        </div>
    );
};

export default Summary;
