class Kissagotchi {
    constructor() {
        this.MAX_STAT = 100;
        
        // Default state
        this.state = {
            hunger: 100,
            happiness: 100,
            energy: 100,
            lastUpdate: Date.now(),
            isSleeping: false
        };

        // DOM elements
        this.bars = {
            hunger: document.getElementById('hunger-bar'),
            happiness: document.getElementById('happiness-bar'),
            energy: document.getElementById('energy-bar')
        };
        
        this.catContainer = document.getElementById('cat');
        this.statusMsg = document.getElementById('status-message');
        this.leftEye = document.querySelector('.left-eye');
        this.rightEye = document.querySelector('.right-eye');
        this.mouth = document.querySelector('path[stroke="#4A4A4A"]'); // The mouth path

        this.init();
    }

    init() {
        this.loadState();
        this.updateBars();
        this.updateFace();
        
        // Event listeners
        document.getElementById('btn-feed').addEventListener('click', () => this.feed());
        document.getElementById('btn-play').addEventListener('click', () => this.play());
        document.getElementById('btn-sleep').addEventListener('click', () => this.toggleSleep());

        // Game loop (updates every 5 seconds)
        setInterval(() => this.gameLoop(), 5000);
        
        // Save state before closing
        window.addEventListener('beforeunload', () => this.saveState());
        
        // Initial greeting
        this.showMessage("Miau!");
    }

    loadState() {
        const saved = localStorage.getItem('kissagotchi_state');
        if (saved) {
            const parsed = JSON.parse(saved);
            this.state = { ...this.state, ...parsed };
            this.calculateOfflineProgress();
        }
    }

    saveState() {
        this.state.lastUpdate = Date.now();
        localStorage.setItem('kissagotchi_state', JSON.stringify(this.state));
    }

    calculateOfflineProgress() {
        const now = Date.now();
        const diffMs = now - this.state.lastUpdate;
        const diffMinutes = Math.floor(diffMs / 60000);
        
        if (diffMinutes > 0) {
            // Stats decrease by roughly 1 point per 2 minutes
            const decrease = Math.floor(diffMinutes / 2);
            
            if (this.state.isSleeping) {
                this.state.energy = Math.min(this.MAX_STAT, this.state.energy + (diffMinutes * 2));
                this.state.hunger -= decrease;
                // Happiness stays same while sleeping
            } else {
                this.state.hunger -= decrease;
                this.state.happiness -= decrease;
                this.state.energy -= decrease;
            }
            
            this.clampStats();
        }
    }

    gameLoop() {
        if (this.state.isSleeping) {
            this.state.energy += 2;
            this.state.hunger -= 0.5;
            
            // Wake up if fully rested
            if (this.state.energy >= this.MAX_STAT) {
                this.toggleSleep();
                this.showMessage("Olen virkeä!");
            }
        } else {
            this.state.hunger -= 0.5;
            this.state.happiness -= 0.5;
            this.state.energy -= 0.3;
        }

        this.clampStats();
        this.updateBars();
        this.updateFace();
        this.saveState();
    }

    clampStats() {
        this.state.hunger = Math.max(0, Math.min(this.MAX_STAT, this.state.hunger));
        this.state.happiness = Math.max(0, Math.min(this.MAX_STAT, this.state.happiness));
        this.state.energy = Math.max(0, Math.min(this.MAX_STAT, this.state.energy));
    }

    updateBars() {
        this.bars.hunger.style.width = `${this.state.hunger}%`;
        this.bars.happiness.style.width = `${this.state.happiness}%`;
        this.bars.energy.style.width = `${this.state.energy}%`;
    }

    updateFace() {
        // Change expression based on stats
        if (this.state.isSleeping) {
            // Closed eyes
            this.leftEye.setAttribute('r', '2');
            this.rightEye.setAttribute('r', '2');
            this.mouth.setAttribute('d', 'M 95 135 L 105 135'); // Neutral mouth
            this.catContainer.className = 'cat-container anim-sleep';
        } else if (this.state.hunger < 30 || this.state.happiness < 30) {
            // Sad face
            this.leftEye.setAttribute('r', '8');
            this.rightEye.setAttribute('r', '8');
            this.mouth.setAttribute('d', 'M 90 140 Q 100 130 110 140'); // Frown
            this.catContainer.className = 'cat-container anim-sad';
        } else {
            // Happy face
            this.leftEye.setAttribute('r', '8');
            this.rightEye.setAttribute('r', '8');
            this.mouth.setAttribute('d', 'M 90 135 Q 95 142 100 135 Q 105 142 110 135'); // Smile
            this.catContainer.className = 'cat-container';
        }
    }

    feed() {
        if (this.state.isSleeping) return this.showMessage("Zzz... en voi syödä nukkuessa.");
        
        this.state.hunger += 20;
        this.state.energy += 5;
        this.clampStats();
        this.updateBars();
        this.updateFace();
        
        this.triggerAnimation('anim-eat');
        this.showMessage("Nam nam! 🐟");
        this.saveState();
    }

    play() {
        if (this.state.isSleeping) return this.showMessage("Zzz... haluan nukkua.");
        if (this.state.energy < 20) return this.showMessage("Olen liian väsynyt leikkimään...");
        if (this.state.hunger < 20) return this.showMessage("Olen liian nälkäinen...");

        this.state.happiness += 20;
        this.state.energy -= 15;
        this.state.hunger -= 10;
        this.clampStats();
        this.updateBars();
        this.updateFace();

        this.triggerAnimation('anim-play');
        this.showMessage("Purrrr! 🧶");
        this.saveState();
    }

    toggleSleep() {
        this.state.isSleeping = !this.state.isSleeping;
        this.updateFace();
        
        if (this.state.isSleeping) {
            this.showMessage("Hyvää yötä! 💤");
        } else {
            this.showMessage("Huomenta! ☀️");
        }
        this.saveState();
    }

    triggerAnimation(className) {
        this.catContainer.classList.remove('anim-eat', 'anim-play', 'anim-sad');
        // Force reflow
        void this.catContainer.offsetWidth;
        this.catContainer.classList.add(className);
        
        setTimeout(() => {
            if (!this.state.isSleeping) {
                this.updateFace(); // restores default idle class
            }
        }, 1500);
    }

    showMessage(text) {
        this.statusMsg.textContent = text;
        this.statusMsg.classList.remove('show');
        void this.statusMsg.offsetWidth; // Force reflow
        this.statusMsg.classList.add('show');
    }
}

// Export for testing, or start app when DOM loads
if (typeof module !== 'undefined' && module.exports) {
    module.exports = Kissagotchi;
} else {
    document.addEventListener('DOMContentLoaded', () => {
        window.game = new Kissagotchi();
    });
}
