// components/VolumeSlider.qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import dataflow 1.0  // For SlintSettings

Slider {
    id: root
    property var settings: SlintSettings

    // Slider properties
    orientation: Qt.Vertical
    width: 50
    from: 0
    to: 100
    value: settings ? settings.volumePreset : 50

    onMoved: {
        if (settings) {
            settings.volumePreset = value;
        }
    }

    handle: Rectangle {
        implicitWidth: 20
        implicitHeight: 20
        color: "gray"
        radius: 10
    }
}

// Slider {
//     id: root
//     property var settings: SlintSettings  // In property

//     from: 0
//     to: 100
//     value: settings.volumePreset
//     onMoved: {
//         MediaActions.mediaChangeVolume(value);
//     }
// }
