import { remark } from "https://esm.sh/remark@14?bundle";
import remarkRehype from "https://esm.sh/remark-rehype@10?bundle";
import rehypeHighlight from "https://esm.sh/rehype-highlight@5?bundle";
import rehypeDomStringify from "https://esm.sh/rehype-dom-stringify@3?bundle";

const processor = remark()
  .use(remarkRehype)
  .data('settings', { fragment: true })
  .use(rehypeHighlight, { ignoreMissing: true, detect: true })
  .use(rehypeDomStringify);

const form = document.getElementById("form");
const submit = document.getElementById("submit");
const stop = document.getElementById("stop");
const provider = document.getElementById("provider");
const prompt = document.getElementById("prompt");
const messages = document.getElementById("messages");
let lastMessageId;
let controller;

stop.addEventListener("click", () => {
  controller.abort();
  resetForm();
});

provider.addEventListener("change", () => {
  lastMessageId = undefined;
  messages.innerHTML = "";
});

prompt.addEventListener("keypress", function (e) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    submit.click();
  }
});

function updatePromptArea() {
  prompt.style.cssText = "height: auto; padding: 0;";
  prompt.style.cssText = "height: " + prompt.scrollHeight + "px;";
}

prompt.addEventListener("input", updatePromptArea);

function appendMessage() {
  const msg = document.createElement("div");
  msg.classList.add("message");

  const bubble = document.createElement("div");
  bubble.classList.add("message-bubble");

  msg.appendChild(bubble);
  messages.appendChild(msg);

  return bubble;
}

form.addEventListener("submit", async function(e) {
  e.preventDefault();

  const trimmedPromptValue = prompt.value.trim();

  if (!trimmedPromptValue) return;

  submit.setAttribute("disabled", "");
  provider.setAttribute("disabled", "");

  prompt.value = "";
  updatePromptArea();

  appendMessage().innerHTML = await processor.process(trimmedPromptValue);
  messages.scrollTop = messages.scrollHeight;

  const params = new URLSearchParams({
    provider: provider.value,
    prompt: trimmedPromptValue,
  });

  switch (provider.value) {
    case "bai": {
      if (lastMessageId)
        params.set("state", lastMessageId);
    } break;
    case "deepai": {
      const messageCount = messages.children.length - 1;

      if (messageCount) {
        const chat = [];

        for (let i = 0; i < messageCount; i++) {
          chat.push({
            role: i % 2 === 0 ? "user" : "assistant",
            content: messages.children[i].innerText,
          });
        }

        params.set("state", JSON.stringify(chat));
      }
    } break;
    case "you": {
      const messageCount = messages.children.length - 1;

      if (messageCount) {
        const chat = [];

        for (let i = 0; i < messageCount - 1; i++) {
          chat.push({
            question: messages.children[i].innerText,
            answer: messages.children[i+1].innerText,
          });
        }

        params.set("state", lastMessageId + JSON.stringify(chat));
      }
    } break;
  }

  controller = new AbortController();
  const res = await fetch(`${location.origin}/api/ask?${params}`, {
    signal: controller.signal,
  });

  if (res.status === 200) {
    const msgId = res.headers.get("msg-id");
    if (msgId) lastMessageId = msgId;
  }

  const bubble = appendMessage();
  const stream = res.body.pipeThrough(new TextDecoderStream());

  submit.style.display = "none";
  stop.style.display = "block";

  let text = "";

  for await (const chunk of stream) {
    text += chunk;

    bubble.innerHTML = (await processor.process(text))
      .value
      .split("\n")
      .map(line => reindent(line, 4, 2))
      .join("\n");

    messages.scrollTop = messages.scrollHeight;
  }

  resetForm();
});

function reindent(line, initial, target) {
  const spaces = " ".repeat(initial);
  let i = 0;

  while (line.slice(i, i + initial) === spaces) {
    i += initial;
  }

  return " ".repeat(target * (i / initial)) + line.slice(i);
}

function resetForm() {
  stop.style.display = "none";
  submit.style.display = "";
  submit.removeAttribute("disabled");
  provider.removeAttribute("disabled");
}
