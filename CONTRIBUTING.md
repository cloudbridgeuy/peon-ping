# Contributing to peon-ping

Want to add a sound pack? We'd love that.

## Add a new sound pack

Sound files are version-controlled in the repo. No external downloads needed.

### 1. Create your pack

```
packs/<name>/
  manifest.json
  sounds/
    SomeSound.wav
    AnotherSound.mp3
    ...
```

Audio formats: WAV, MP3, or OGG. Keep files small (game sound effects are ideal).

### 2. Write the manifest

Map your sounds to categories. See `packs/peon/manifest.json` for the full example:

```json
{
  "name": "my_pack",
  "display_name": "My Character",
  "categories": {
    "greeting": {
      "sounds": [
        { "file": "Hello.mp3", "line": "Hello there" }
      ]
    },
    "acknowledge": {
      "sounds": [
        { "file": "OnIt.mp3", "line": "On it" }
      ]
    },
    "complete": {
      "sounds": [
        { "file": "Done.mp3", "line": "Done" }
      ]
    },
    "error": {
      "sounds": [
        { "file": "Oops.mp3", "line": "Oops" }
      ]
    },
    "permission": {
      "sounds": [
        { "file": "NeedHelp.mp3", "line": "Need your help" }
      ]
    },
    "resource_limit": {
      "sounds": [
        { "file": "Blocked.mp3", "line": "Blocked" }
      ]
    },
    "annoyed": {
      "sounds": [
        { "file": "StopIt.mp3", "line": "Stop it" }
      ]
    }
  }
}
```

**Categories explained:**

| Category | When it plays |
|---|---|
| `greeting` | Session starts (`$ claude`) |
| `acknowledge` | Claude acknowledges a task |
| `complete` | Claude finishes and is idle |
| `error` | Something fails |
| `permission` | Claude needs tool approval |
| `resource_limit` | Resource limits hit |
| `annoyed` | User spams prompts (3+ in 10 seconds) |

Not every category is required — just include the ones you have sounds for.

### 3. Add your pack to install.sh

Add your pack name to the `PACKS` variable:

```bash
PACKS="peon ra2_soviet_engineer my_pack"
```

### 4. Bump the version

We use [semver](https://semver.org/). Edit the `VERSION` file in the repo root:

- **New sound pack** → bump the patch version (e.g. `1.0.0` → `1.0.1`)
- **New feature** (new hook event, config option) → bump the minor version (e.g. `1.0.1` → `1.1.0`)
- **Breaking change** (config format change, removed feature) → bump the major version

Users with an older version will see an update notice on session start.

### 5. Submit a PR

That's it. We'll review and merge.

## Automate pack creation

Have a single audio file with all your character's quotes? You can auto-transcribe and split it:

1. Copy `.env.example` to `.env` and add your [Deepgram API key](https://console.deepgram.com) (or use [Whisper](https://github.com/openai/whisper) locally)
2. Transcribe with word-level timestamps:

```bash
# Option A: Deepgram (cloud, fast)
source .env
curl --http1.1 -X POST \
  "https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&utterances=true&utt_split=0.8" \
  -H "Authorization: Token $DEEPGRAM_API_KEY" \
  -H "Content-Type: audio/mpeg" \
  --data-binary @your_audio.mp3 -o transcription.json

# Option B: Whisper (local, free)
pip install openai-whisper
whisper your_audio.mp3 --model base --language en --output_format json --word_timestamps True --output_dir .
```

3. Use the timestamps from the JSON to cut individual clips with ffmpeg:

```bash
ffmpeg -i your_audio.mp3 -ss 0.0 -to 1.5 -c copy packs/my_pack/sounds/Quote1.mp3 -y
ffmpeg -i your_audio.mp3 -ss 2.0 -to 4.8 -c copy packs/my_pack/sounds/Quote2.mp3 -y
# ... repeat for each quote
```

4. Map the clips to categories in `manifest.json` and you're done.

This is how the StarCraft Battlecruiser and Kerrigan packs were created — one source audio file, transcribed, split, and mapped in minutes.

## Pack ideas

We'd love to see these (or anything else):

- Human Peasant ("Job's done!")
- Night Elf Wisp
- Undead Acolyte
- Protoss Probe
- SCV
- GLaDOS
- Navi ("Hey! Listen!")
- Clippy
