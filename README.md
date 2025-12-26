# Andi Rusty Bot

A powerful AI assistant built with Rust, Actix Web, and OpenRouter API.

## Features

- ⚡ Lightning-fast responses powered by Rust
- 🤖 AI-powered using OpenRouter API
- 🌐 Modern web interface with parallax effects
- 🔒 Secure and reliable

## Local Testing

### Prerequisites

1. **Get an OpenRouter API Key**
   - Visit https://openrouter.ai/keys
   - Sign up or log in
   - Create a new API key
   - Copy your API key

2. **Create a `.env` file** in the project root:
   ```bash
   # Create .env file
   echo "OPENROUTER_API_KEY=your_api_key_here" > .env
   ```
   
   Or manually create `.env` with:
   ```
   OPENROUTER_API_KEY=sk-or-v1-...
   ```

### Running the Server

1. **Start the server:**
   ```bash
   cargo run
   ```

2. **You should see:**
   ```
   🚀 Starting server on http://0.0.0.0:8080
   📡 POST endpoint: http://0.0.0.0:8080/completion
   🌐 Static files served from /static directory
   ```

3. **Open your browser:**
   - Navigate to `http://localhost:8080`
   - You should see the landing page with parallax effects

### Testing the Web Interface

1. Scroll down to the **Features** section
2. You'll see an input field with a submit button
3. Type a question (e.g., "What is Rust?")
4. Click "Ask" or press Enter
5. Wait for the AI response to appear below

### Testing the API Directly

You can test the API endpoint using `curl`:

```bash
curl -X POST http://localhost:8080/completion \
  -H "Content-Type: application/json" \
  -d '{"question": "Why do leaves change colour in autumn?"}'
```

Or using `httpie`:

```bash
http POST http://localhost:8080/completion question="What is Rust programming language?"
```

Or test in your browser's console:
```javascript
fetch('/completion', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ question: 'Hello, how are you?' })
})
.then(r => r.json())
.then(console.log);
```

### Troubleshooting

**Server won't start:**
- Make sure you have Rust installed: `rustc --version`
- Check that `.env` file exists and has `OPENROUTER_API_KEY` set
- Verify the port 8080 is not already in use

**API returns errors:**
- Verify your OpenRouter API key is correct
- Check that you have credits/balance on OpenRouter
- Look at server logs for detailed error messages

**Images not loading:**
- Make sure `rusty.jpg` and `coderbot.jpg` exist in the `static/` directory
- Check browser console for 404 errors

## API Endpoint

### POST `/completion`

Send a question to get an AI response.

**Request:**
```json
{
  "question": "Why do leaves change colour in autumn?"
}
```

**Response:**
```json
{
  "answer": "..."
}
```

## Deployment on Render.com

1. Push your code to GitHub
2. Create a new Web Service on Render.com
3. Connect your GitHub repository
4. Configure the service:
   - **Environment**: Rust
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/openrouter_llm_chain_demo`
5. Add environment variable:
   - `OPENROUTER_API_KEY`: Your OpenRouter API key
6. Deploy!

The `PORT` environment variable is automatically set by Render.com.

## Project Structure

```
.
├── src/
│   └── main.rs          # Actix web server and API endpoints
├── static/
│   ├── index.html       # Landing page with chat interface
│   ├── rusty.jpg        # Static assets
│   └── coderbot.jpg
├── Cargo.toml           # Rust dependencies
└── render.yaml          # Render.com configuration
```

## Technologies

- **Rust** - Systems programming language
- **Actix Web** - Web framework
- **OpenRouter** - AI model API
- **llm-chain** - LLM chain orchestration

