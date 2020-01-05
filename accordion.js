function makeWave(ac) {
    const nPartials = 32
    const sines = new Float32Array(nPartials)
    const cosines = new Float32Array(nPartials)

    sines[0] = 0;
    let amplitude = 0.03;
    const decay = 2
    for (i=1; i<nPartials; i++) {
        sines[i] = amplitude
        amplitude /= decay
    }

    cosines.fill(0)

    return ac.createPeriodicWave(
        sines, cosines, {disableNormalization: true})
}

const ac = new AudioContext()
const wave = makeWave(ac);
const pitchesPlaying = new Map()

function togglePitch(evt) {
    if (evt.target.hasAttribute('data-freq')) {
        let frequency = evt.target.getAttribute('data-freq')
        if (pitchesPlaying.has(frequency)) {
            let osc = pitchesPlaying.get(frequency)
            osc.stop()
            pitchesPlaying.delete(frequency)
            evt.target.classList.remove("playing")
        }
        else {
            let osc = ac.createOscillator()
            osc.setPeriodicWave(wave)
            osc.frequency.value = parseInt(frequency, 10)
            osc.connect(ac.destination)
            pitchesPlaying.set(frequency, osc)
            osc.start()
            evt.target.classList.add("playing")
        }
    }
}

for (container of document.getElementsByClassName("accordion")) {
    container.addEventListener('click', togglePitch)
}
