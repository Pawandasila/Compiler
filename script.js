const compileBtn = document.getElementById("compile-btn");
const rulesBtn = document.getElementById("rules-btn");
const examplesBtn = document.getElementById("examples-btn");
const sourceCode = document.getElementById("source-code");
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

// Compile function
async function compile() {
  const code = sourceCode.value.trim();

  if (!code) {
    showError("Please enter some code to compile.");
    return;
  }

  // Set loading state
  setLoadingState(true);
  hideError();

  resultOutput.textContent = "🔄 Compiling your code...";
  bytecodeOutput.textContent = "⚙️  Generating bytecode...";

  try {
    const response = await fetch("http://127.0.0.1:8080/compile", {
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
      showError(data.error);
      resultOutput.textContent =
        "❌ Compilation failed. Check the error below.";
      bytecodeOutput.textContent =
        "❌ No bytecode generated due to compilation error.";
    } else {
      resultOutput.textContent = `✅ Result: ${data.result || "No output"}`;
      bytecodeOutput.textContent =
        data.bytecode?.join("\n") || "No bytecode generated";

      // Add success animation
      resultOutput.style.animation = "none";
      resultOutput.offsetHeight; // Trigger reflow
      resultOutput.style.animation = "fadeIn 0.5s ease-out";
    }
  } catch (error) {
    showError(
      `Connection error: ${error.message}\n\nPlease ensure the compiler server is running on http://127.0.0.1:8080`
    );
    resultOutput.textContent = "❌ Failed to connect to compiler server.";
    bytecodeOutput.textContent = "❌ Server connection unavailable.";
  } finally {
    setLoadingState(false);
  }
}
function setLoadingState(loading) {
  if (loading) {
    compileBtn.disabled = true;
    spinner.style.display = "inline-block";
    compileBtn.querySelector("span").textContent = "⏳ Compiling...";
  } else {
    compileBtn.disabled = false;
    spinner.style.display = "none";
    compileBtn.querySelector("span").textContent = "⚡ Compile & Run";
  }
}

function showError(message) {
  errorPanel.style.display = "block";
  errorContent.textContent = message;

  // Add shake animation
  errorPanel.style.animation = "none";
  errorPanel.offsetHeight; // Trigger reflow
  errorPanel.style.animation = "shake 0.5s ease-in-out";
}

function hideError() {
  errorPanel.style.display = "none";
}

// Load an example into the editor
function loadExample() {
  const example = examples[currentExampleIndex];
  sourceCode.value = example.code;

  // Increment for next example
  currentExampleIndex = (currentExampleIndex + 1) % examples.length;
}

// Toggle rules modal
function toggleModal(show) {
  modalOverlay.style.display = show ? "block" : "none";
}

// Event listeners
compileBtn.addEventListener("click", compile);

rulesBtn.addEventListener("click", () => {
  toggleModal(true);
});

closeModal.addEventListener("click", () => {
  toggleModal(false);
});

examplesBtn.addEventListener("click", () => {
  loadExample();
});

// Add keyboard shortcut for compilation
document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
    e.preventDefault();
    compile();
  }
});

// Close modal when clicking outside
modalOverlay.addEventListener("click", (e) => {
  if (e.target === modalOverlay) {
    toggleModal(false);
  }
});

// Initialize with focus in the editor
sourceCode.focus();
