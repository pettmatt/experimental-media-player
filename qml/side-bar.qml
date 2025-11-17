import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dataflow 1.0  // For SlintPlaylist

Item {
    id: root
    property var list: list  // List of SlintPlaylist

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        // Iterate over the list of playlists
        Repeater {
            model: list

            // Playlist component
            Playlist {
                Layout.fillWidth: true
                Layout.preferredHeight: 30
                data: modelData
            }
        }
    }
}
