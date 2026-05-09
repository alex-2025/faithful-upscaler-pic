const { open } = window.__TAURI__.dialog;
const { invoke } = window.__TAURI__.core;

const state = {
  inputPath: null,
  targetMegapixels: 7,
  busy: false
};

const elements = {
  pickImage: document.querySelector("#pick-image"),
  run: document.querySelector("#run"),
  imageName: document.querySelector("#image-name"),
  imageMeta: document.querySelector("#image-meta"),
  targetMeta: document.querySelector("#target-meta"),
  outputPath: document.querySelector("#output-path"),
  status: document.querySelector("#status"),
  choices: Array.from(document.querySelectorAll(".choice"))
};

function setStatus(message, type = "idle") {
  elements.status.textContent = message;
  elements.status.dataset.type = type;
}

function setBusy(busy) {
  state.busy = busy;
  elements.pickImage.disabled = busy;
  elements.run.disabled = busy || !state.inputPath;

  for (const choice of elements.choices) {
    choice.disabled = busy;
  }
}

function formatMegapixels(value) {
  return `${value.toFixed(2)} MP`;
}

function updateChoiceUI() {
  for (const choice of elements.choices) {
    const active = Number(choice.dataset.target) === state.targetMegapixels;
    choice.classList.toggle("active", active);
  }

  elements.targetMeta.textContent = `${state.targetMegapixels}MP 保真放大`;
}

async function pickImage() {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png", "bmp", "webp"]
      }
    ]
  });

  if (!selected || Array.isArray(selected)) {
    return;
  }

  setBusy(true);
  setStatus("正在读取图片信息...", "work");

  try {
    const info = await invoke("inspect_image", { path: selected });
    state.inputPath = selected;
    elements.imageName.textContent = info.path.split(/[/\\]/).pop();
    elements.imageMeta.textContent =
      `${info.width} x ${info.height}  ·  ${formatMegapixels(info.megapixels)}  ·  ${info.format}`;
    elements.outputPath.textContent = "处理完成后会自动生成新文件";
    setStatus("图片已就绪，可以开始处理", "ok");
  } catch (error) {
    state.inputPath = null;
    elements.imageName.textContent = "尚未选择图片";
    elements.imageMeta.textContent = "支持 JPG、PNG、BMP、WEBP";
    elements.outputPath.textContent = "会自动保存在原图同目录";
    setStatus(`读取失败：${error}`, "error");
  } finally {
    setBusy(false);
  }
}

async function runUpscale() {
  if (!state.inputPath || state.busy) {
    return;
  }

  setBusy(true);
  setStatus("正在放大图片，请稍候...", "work");
  elements.outputPath.textContent = "处理中...";

  try {
    const result = await invoke("upscale_image", {
      inputPath: state.inputPath,
      targetMegapixels: state.targetMegapixels
    });

    elements.outputPath.textContent = result.output_path;
    setStatus(
      `完成：${result.width} x ${result.height} · ${formatMegapixels(result.megapixels)}`,
      "ok"
    );
  } catch (error) {
    elements.outputPath.textContent = "未生成结果";
    setStatus(`处理失败：${error}`, "error");
  } finally {
    setBusy(false);
  }
}

elements.pickImage.addEventListener("click", pickImage);
elements.run.addEventListener("click", runUpscale);

for (const choice of elements.choices) {
  choice.addEventListener("click", () => {
    if (state.busy) {
      return;
    }

    state.targetMegapixels = Number(choice.dataset.target);
    updateChoiceUI();
  });
}

updateChoiceUI();
setBusy(false);
