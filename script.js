let editor; // CodeMirror instance
let errorMarkers = [];

document.addEventListener("DOMContentLoaded", function() {
  // Initialize CodeMirror
  editor = CodeMirror.fromTextArea(document.getElementById("source-code"), {
    mode: "text/x-csrc", // Similar enough to our custom language
    theme: "dracula",
    lineNumbers: true,
    lineWrapping: true,
    tabSize: 2,
    indentWithTabs: false,
    autoCloseBrackets: true,
    matchBrackets: true,
    highlightSelectionMatches: {showToken: /\w/, annotateScrollbar: true},
    extraKeys: {
      "Ctrl-Enter": compile,
      "Cmd-Enter": compile,
      "Ctrl-Space": "autocomplete",
    }
  });

  // Define custom syntax highlighting for our language
  defineCustomSyntax();

  // Set initial height
  editor.setSize("100%", "100%");

  // Elements
  const compileBtn = document.getElementById("compile-btn");
  const rulesBtn = document.getElementById("rules-btn");
  const examplesBtn = document.getElementById("examples-btn");
  const resultOutput = document.getElementById("result-output");
  const bytecodeOutput = document.getElementById("bytecode-output");
  const errorPanel = document.getElementById("error-panel");
  const errorContent = document.getElementById("error-content");
  const spinner = document.getElementById("spinner");
  const modalOverlay = document.getElementById("modal-overlay");
  const closeModal = document.getElementById("close-modal");

  // Example codes
  const examples = [
    {
      name: "Basic Arithmetic",
      code: `// Basic arithmetic operations
int a = 15;
int b = 4;

int sum = a + b;        // 19
int difference = a - b; // 11
int product = a * b;    // 60
int quotient = a / b;   // 3

sum;`,
    },
    {
      name: "Complex Expression",
      code: `// Complex mathematical expression
int x = 8;
int y = 3;
int z = 2;

int result = (x + y) * z - x / y;
result;`,
    },
    {
      name: "Float Operations",
      code: `// Working with floating-point numbers
float pi = 3.14159;
float radius = 5.0;

float area = pi * radius * radius;
area;`,
    },
    {
      name: "Multiple Variables",
      code: `// Multiple variable calculations
int base = 10;
int height = 15;
int triangleArea = base * height / 2;

int rectangleLength = 12;
int rectangleWidth = 8;
int rectangleArea = rectangleLength * rectangleWidth;

triangleArea + rectangleArea;`,
    },
  ];

  let currentExampleIndex = 0;

  // Event listeners
  compileBtn.addEventListener("click", compile);
  rulesBtn.addEventListener("click", showRulesModal);
  examplesBtn.addEventListener("click", showNextExample);
  closeModal.addEventListener("click", hideModal);
  modalOverlay.addEventListener("click", function(event) {
    if (event.target === modalOverlay) hideModal();
  });

  // Function to define custom syntax highlighting for our language
  function defineCustomSyntax() {
    // Add custom keywords for our language
    CodeMirror.defineMode("custom-language", function(config) {
      const baseMode = CodeMirror.getMode(config, "text/x-csrc");
      
      return {
        startState: function() {
          return {
            baseState: CodeMirror.startState(baseMode)
          };
        },
        token: function(stream, state) {
          // Keywords specific to our custom language
          if (stream.match(/\b(int|float)\b/)) {
            return "keyword";
          }
          
          return baseMode.token(stream, state.baseState);
        }
      };
    });
    
    // Set our custom mode
    editor.setOption("mode", "custom-language");
  }

  // Compile function
  async function compile() {
    const code = editor.getValue().trim();

    if (!code) {
      showError("Please enter some code to compile.");
      return;
    }

    // Clear previous error markers
    clearErrorMarkers();
    
    // Set loading state
    setLoadingState(true);
    hideError();

    resultOutput.textContent = "🔄 Compiling your code...";
    bytecodeOutput.textContent = "⚙️ Generating bytecode...";

    try {
      const response = await fetch("/compile", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          source: code,
          language: "custom",
        }),
      });

      const data = await response.json();

      if (data.error) {
        showErrorWithHighlighting(data.error);
        resultOutput.textContent = "❌ Compilation failed. Check the error below.";
        bytecodeOutput.textContent = "❌ No bytecode generated due to compilation error.";
      } else {
        resultOutput.innerHTML = `<span class="success-icon">✅</span> Result: <span class="result-value">${data.result || "No output"}</span>`;
        
        // Format bytecode with syntax highlighting
        bytecodeOutput.innerHTML = formatBytecode(data.bytecode);
        
        // Animate result
        animateResult();
      }
    } catch (error) {
      showError(`Network error: ${error.message}`);
      resultOutput.textContent = "❌ Failed to connect to the server.";
      bytecodeOutput.textContent = "❌ Connection error.";
    } finally {
      setLoadingState(false);
    }
  }

  // Format bytecode with syntax highlighting
  function formatBytecode(bytecode) {
    if (!bytecode || bytecode.length === 0) {
      return "<em>No bytecode generated</em>";
    }

    // Create HTML for bytecode with syntax highlighting
    return bytecode.map((line, index) => {
      // Highlight different parts of the bytecode
      return `<div class="bytecode-line">
        <span class="bytecode-index">${index.toString().padStart(2, '0')}</span>
        ${highlightBytecode(line)}
      </div>`;
    }).join("");
  }

  // Highlight different parts of bytecode instructions
  function highlightBytecode(instruction) {
    // Replace specific patterns with highlighted spans
    return instruction
      .replace(/Push\(([^)]+)\)/g, 'Push(<span class="bytecode-value">$1</span>)')
      .replace(/(Add|Subtract|Multiply|Divide|Negate)/g, '<span class="bytecode-op">$1</span>')
      .replace(/(Jump|JumpIfFalse)\((\d+)\)/g, '<span class="bytecode-flow">$1</span>(<span class="bytecode-number">$2</span>)')
      .replace(/(Load|Store)Variable\("([^"]+)"\)/g, '<span class="bytecode-var">$1Variable</span>("$2")');
  }

  // Show error with code highlighting
  function showErrorWithHighlighting(errorMessage) {
    showError(errorMessage);
    
    // Try to extract line and column information
    const lineMatch = errorMessage.match(/at (\d+):(\d+)/);
    
    if (lineMatch) {
      const line = parseInt(lineMatch[1], 10) - 1; // CodeMirror lines are 0-based
      const col = parseInt(lineMatch[2], 10) - 1;
      
      // Highlight the error line
      const marker = editor.addLineClass(line, "background", "error-line");
      errorMarkers.push(marker);
      
      // Mark the specific error position if possible
      if (!isNaN(col)) {
        const from = { line: line, ch: col };
        const to = { line: line, ch: col + 1 };
        const mark = editor.markText(from, to, { className: "error-highlight" });
        errorMarkers.push(mark);
      }
      
      // Scroll to the error
      editor.scrollIntoView({ line: line, ch: 0 }, 200);
    }
  }

  // Clear error markers
  function clearErrorMarkers() {
    errorMarkers.forEach(marker => {
      if (marker.clear) {
        marker.clear();
      } else {
        editor.removeLineClass(marker, "background", "error-line");
      }
    });
    errorMarkers = [];
  }

  // Animate result
  function animateResult() {
    const resultValue = document.querySelector(".result-value");
    if (resultValue) {
      resultValue.classList.add("result-highlight");
      setTimeout(() => {
        resultValue.classList.remove("result-highlight");
      }, 2000);
    }
  }

  // Show error
  function showError(message) {
    errorPanel.style.display = "block";
    errorContent.textContent = message;
    
    // Scroll to error panel
    setTimeout(() => {
      errorPanel.scrollIntoView({ behavior: "smooth" });
    }, 100);
  }

  // Hide error
  function hideError() {
    errorPanel.style.display = "none";
  }

  // Set loading state
  function setLoadingState(isLoading) {
    if (isLoading) {
      spinner.style.display = "block";
      compileBtn.disabled = true;
    } else {
      spinner.style.display = "none";
      compileBtn.disabled = false;
    }
  }

  // Show rules modal
  function showRulesModal() {
    modalOverlay.style.display = "flex";
  }

  // Hide modal
  function hideModal() {
    modalOverlay.style.display = "none";
  }

  // Show next example
  function showNextExample() {
    currentExampleIndex = (currentExampleIndex + 1) % examples.length;
    const example = examples[currentExampleIndex];
    editor.setValue(example.code);
    
    // Show a toast notification
    showToast(`Example: ${example.name}`);
  }

  // Show toast notification
  function showToast(message) {
    // Create toast element if it doesn't exist
    let toast = document.getElementById("toast");
    if (!toast) {
      toast = document.createElement("div");
      toast.id = "toast";
      document.body.appendChild(toast);
    }
    
    // Set message and show
    toast.textContent = message;
    toast.className = "toast-visible";
    
    // Hide after 3 seconds
    setTimeout(() => {
      toast.className = "";
    }, 3000);
  }

  // Enable keyboard shortcuts
  document.addEventListener("keydown", function(event) {
    // Ctrl+Enter or Cmd+Enter to compile
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      compile();
    }
  });

  // Add styles for new elements that are created dynamically
  addDynamicStyles();
  
  function addDynamicStyles() {
    const styleEl = document.createElement("style");
    styleEl.textContent = `
      /* Bytecode styling */
      .bytecode-line {
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        line-height: 1.5;
        margin-bottom: 2px;
      }
      .bytecode-index {
        color: #6272a4;
        display: inline-block;
        width: 30px;
        margin-right: 8px;
      }
      .bytecode-op {
        color: #ff79c6;
        font-weight: bold;
      }
      .bytecode-value {
        color: #f1fa8c;
      }
      .bytecode-flow {
        color: #bd93f9;
      }
      .bytecode-var {
        color: #8be9fd;
      }
      .bytecode-number {
        color: #bd93f9;
      }
      
      /* Result animation */
      .success-icon {
        display: inline-block;
        animation: pulse 1.5s infinite;
      }
      .result-value {
        font-weight: bold;
        color: #50fa7b;
      }
      .result-highlight {
        animation: highlight 1.5s ease-out;
      }
      
      /* Toast notification */
      #toast {
        visibility: hidden;
        min-width: 250px;
        background-color: rgba(80, 250, 123, 0.9);
        color: #282a36;
        text-align: center;
        border-radius: 10px;
        padding: 16px;
        position: fixed;
        z-index: 1000;
        left: 50%;
        bottom: 30px;
        transform: translateX(-50%);
        font-weight: bold;
        box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
      }
      
      #toast.toast-visible {
        visibility: visible;
        animation: fadeInOut 3s;
      }
      
      @keyframes fadeInOut {
        0% { opacity: 0; transform: translateX(-50%) translateY(20px); }
        10% { opacity: 1; transform: translateX(-50%) translateY(0); }
        90% { opacity: 1; transform: translateX(-50%) translateY(0); }
        100% { opacity: 0; transform: translateX(-50%) translateY(-20px); }
      }
      
      @keyframes highlight {
        0% { background-color: rgba(80, 250, 123, 0.2); }
        100% { background-color: transparent; }
      }
    `;
    document.head.appendChild(styleEl);
  }
});
