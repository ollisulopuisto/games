GAMES := zookeeper bubbles lumines gravitris music_editor kissagotchi
VERSION := v26.4.30.256

# Define icons for each game (img tag or span for emoji)
ICON_zookeeper := <img src="assets/icon.png" alt="Zookeeper">
ICON_bubbles   := <img src="assets/bubbles_icon.png" alt="Bubbles">
ICON_lumines   := <span style="font-size: 5rem; margin-bottom: 1rem;">🔲</span>
ICON_gravitris := <span style="font-size: 5rem; margin-bottom: 1rem;">🧲</span>
ICON_music_editor := <span style="font-size: 5rem; margin-bottom: 1rem;">🎵</span>
ICON_jetpac    := <img src="assets/1f680.png" alt="Jetpac">
ICON_voxeldash := <span style="font-size: 5rem; margin-bottom: 1rem;">📦</span>
ICON_kissagotchi := <span style="font-size: 5rem; margin-bottom: 1rem;">🐱</span>

# Define display names
NAME_zookeeper := Zookeeper
NAME_bubbles   := Bubbles
NAME_lumines   := Lumines
NAME_gravitris := Gravitris
NAME_music_editor := Music Editor
NAME_jetpac    := Jetpac
NAME_voxeldash := VoxelDash
NAME_kissagotchi := Kissagotchi

# Define control descriptions
CTRL_zookeeper := Swap gems with mouse / touch.<br><b>P</b> to Pause.
CTRL_bubbles   := Arrow keys / WASD to move &amp; jump.<br>SPACE to blow bubbles. <b>P</b> to Pause.
CTRL_lumines   := Arrow keys / WASD to move &amp; rotate.<br>SPACE to drop. <b>P</b> to Pause.
CTRL_gravitris := Ambient Tetris. Arrows/WASD to move.<br>SPACE to drop. <b>P</b> to Pause, <b>M</b> to Mute.
CTRL_music_editor := Procedural music pattern editor.<br><b>Randomize</b> to generate new patterns.
CTRL_kissagotchi := Pidä huolta kissasta.<br>Syötä, leiki ja nukuta.

.PHONY: all setup build test lint clean rename index.html $(GAMES)

all: build

setup:
	@for game in $(GAMES); do \
		echo "Setting up $$game..."; \
		$(MAKE) -C games/$$game setup; \
	done

index.html: index.template.html
	@echo "Generating index.html..."
	@rm -f .grid.tmp
	@$(foreach game,$(GAMES), \
		echo '        <a href="$(game)/" class="card">' >> .grid.tmp; \
		echo '            $(subst ','\'',$(ICON_$(game)))' >> .grid.tmp; \
		echo '            <div class="title">$(NAME_$(game))</div>' >> .grid.tmp; \
		echo '            <div class="controls">$(CTRL_$(game))</div>' >> .grid.tmp; \
		echo '        </a>' >> .grid.tmp; \
	)
	@sed -e '/{{GAMES_GRID}}/{r .grid.tmp' -e 'd;}' \
	     -e 's/{{VERSION}}/$(VERSION)/g' \
	     index.template.html > index.html
	@rm -f .grid.tmp

build: index.html
	@mkdir -p docs/assets
	cp index.html docs/
	cp scripts/platform.js docs/assets/platform.js
	cp assets/icon.png docs/assets/icon.png
	cp assets/bubbles_icon.png docs/assets/bubbles_icon.png
	@for game in $(GAMES); do \
		if [ -z "$(BUILD_GAMES)" ] || echo " $(BUILD_GAMES) " | grep -q " $$game "; then \
			echo "Building $$game..."; \
			$(MAKE) -C games/$$game build || exit 1; \
		elif [ -d games/$$game/dist ]; then \
			echo "Skipping build for $$game (using existing dist)..."; \
		else \
			echo "Cache miss for $$game dist; building $$game..."; \
			$(MAKE) -C games/$$game build || exit 1; \
		fi; \
		mkdir -p docs/$$game || exit 1; \
		rm -rf docs/$$game/* || exit 1; \
		cp scripts/platform.js docs/$$game/platform.js || exit 1; \
		cp -r games/$$game/dist/* docs/$$game/ || exit 1; \
	done

test:
	cargo test --workspace
	@for game in $(GAMES); do \
		if [ -f games/$$game/Makefile ]; then \
			$(MAKE) -C games/$$game test; \
		fi; \
	done

lint:
	@for game in $(GAMES); do \
		$(MAKE) -C games/$$game lint; \
	done

clean:
	cargo clean
	@for game in $(GAMES); do \
		rm -rf games/$$game/dist; \
	done
	rm -rf docs
	rm -f index.html

rename:
	@echo "Renaming repository to 'games'..."
	gh repo edit ollisulopuisto/zookeeper-wasm --name games
	git remote set-url origin https://github.com/ollisulopuisto/games.git
	@echo "Repository renamed. Please update any hardcoded URLs."
