import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dataflow 1.0  // For MediaActions, SlintState, SlintTimeLine

Item {
    id: root
    property var playing: SlintTimeLine  // In property

    // Timer to update track position
    Timer {
        id: timer
        interval: 1000
        running: SlintState.timelineLength > 0
        repeat: true
        onTriggered: {
            MediaActions.mediaGetTrackPosition();
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        // First row: Buttons
        RowLayout {
            spacing: 5

            Button {
                text: qsTr("Mix")
                onClicked: MediaActions.mediaMix()
            }

            Button {
                text: qsTr("Prev")
                onClicked: MediaActions.mediaChange(-1)
            }

            Button {
                text: qsTr("Start")
                onClicked: MediaActions.mediaToggle()
            }

            Button {
                text: qsTr("Next")
                onClicked: MediaActions.mediaChange(1)
            }

            Button {
                text: qsTr("Loop")
                onClicked: MediaActions.mediaLoop()
            }

            // Volume button and slider
            Item {
                Layout.alignment: Qt.AlignRight

                // Volume button
                Button {
                    id: volumeButton
                    text: qsTr("Volume")
                    onClicked: volumeSlider.visible = !volumeSlider.visible
                }

                // Volume slider (initially hidden)
                VolumeSlider {
                    id: volumeSlider
                    visible: false
                    y: volumeButton.y - height
                    settings: SlintState.settings
                }
            }
        }

        // Second row: Track position slider
        RowLayout {
            spacing: 5

            // Current time
            Label {
                text: Math.floor(playing.current / 60) + ":" + (playing.current % 60 < 10 ? "0" : "") + playing.current % 60
            }

            // Track position slider
            Slider {
                id: trackSlider
                Layout.fillWidth: true
                from: 0
                to: playing.length
                value: playing.current
                onMoved: {
                    MediaActions.mediaChangeTrackPosition(value);
                }
            }

            // Total time
            Label {
                text: Math.floor(playing.length / 60) + ":" + (playing.length % 60 < 10 ? "0" : "") + playing.length % 60
            }
        }
    }
}
