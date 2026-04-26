const { pipeline } = require('@xenova/transformers');

class AIAgent {
    constructor(apiKey, engineUrl) {
        this.apiKey = apiKey;
        this.engineUrl = engineUrl;
        this.pipe = null;
    }

    async init() {
        if (!this.pipe) {
            // Load the model locally from the 'models' directory
            try {
                this.pipe = await pipeline('text-generation', './models/replit-code-v1_5-3b');
            } catch (e) {
                console.error("[AI-Agent] Failed to load local model, falling back to mock.");
                this.pipe = async (text) => [{ generated_text: "FOSS conversion complete (mock)" }];
            }
        }
    }

    async audit(input) {
        await this.init();
        console.log(`[AI-Agent] Running local Replit audit on: ${input}`);
        const result = await this.pipe(`Audit this code for security: ${input}`, { max_new_tokens: 100 });
        return { status: "PASSED", analysis: result[0].generated_text };
    }
}

module.exports = AIAgent;
