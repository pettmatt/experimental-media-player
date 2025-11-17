import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

import "dataflow"  // For SlintMediaFile, SlintPlaylist, MediaActions, MainView

// Define a custom delegate for media items
Item {
    id: root
    clip: true

    // Expose properties
    property var list: list  // List of SlintMediaFile
    property var playlists: playlists  // List of SlintPlaylist
    property var mainViewId: MainView.index  // MainView index

    // Main scrollable list
    ScrollView {
        anchors.fill: parent

        ListView {
            id: listView
            model: list
            spacing: 5
            clip: true

            delegate: Item {
                width: ListView.view.width
                height: 30

                // TouchArea for click handling
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        MediaActions.mediaStart(modelData.id);
                    }
                }

                // Background rectangle
                Rectangle {
                    anchors.fill: parent
                    color: Palette.secondaryBackground  // Assuming Palette is exposed

                    // Horizontal layout for media info
                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 5
                        spacing: 10

                        Label {
                            text: modelData.name
                            Layout.fillWidth: true
                        }

                        Label {
                            text: modelData.artist
                            Layout.fillWidth: true
                        }

                        Label {
                            text: modelData.playing ? "[1]" : "[0]"
                        }
                    }
                }
            }
        }
    }
}
