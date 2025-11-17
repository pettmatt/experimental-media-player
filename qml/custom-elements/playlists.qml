import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dataflow 1.0  // For SlintPlaylist

Item {
    id: root
    property var data: SlintPlaylist  // In property

    RowLayout {
        spacing: 10
        anchors.fill: parent

        // Image
        Image {
            Layout.preferredWidth: 50
            Layout.preferredHeight: 50
            fillMode: Image.PreserveAspectFit
            source: data.image_url
        }

        // Vertical layout for text
        ColumnLayout {
            spacing: 5

            // Playlist name
            Label {
                text: data.name
                font.pixelSize: 14
            }

            // Source info
            Label {
                text: data.sources.length > 1 ? qsTr("Source: mixed (%1)").arg(data.sources.length) : qsTr("Source: %1").arg(data.sources[0])
                font.pixelSize: 12
            }
        }
    }
}
