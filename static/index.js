const form = document.getElementById("form");
const submit = document.getElementById("submit");
const provider = document.getElementById("provider");
const prompt = document.getElementById("prompt");
const messages = document.getElementById("messages");
let lastMessageId;

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

    appendMessage().innerText = trimmedPromptValue;
    messages.scrollTop = messages.scrollHeight;

    const params = new URLSearchParams({
        provider: provider.value,
        prompt: trimmedPromptValue,
    });

    switch (provider.value) {
        case "bai":
            if (lastMessageId)
                params.set("state", lastMessageId);
            break;
        case "you":
            const messageCount = messages.children.length - 1;

            if (messageCount !== 0) {
                const chat = [];

                for (let i = 0; i < messages.children.length - 1; i++) {
                    chat.push({
                        question: messages.children[i].innerText,
                        answer: messages.children[i+1].innerText,
                    });
                }

                params.set("state", JSON.stringify(chat));
            }
            break;
    }

    const res = await fetch(`${location.origin}/api/ask?${params}`);

    if (res.status === 200) {
        const msgId = res.headers.get("msg-id");
        if (msgId) lastMessageId = msgId;
    }

    const bubble = appendMessage();
    const stream = res.body.pipeThrough(new TextDecoderStream());

    for await (const chunk of stream) {
        bubble.innerText += chunk;
        messages.scrollTop = messages.scrollHeight;
    }

    submit.removeAttribute("disabled");
    provider.removeAttribute("disabled");
});
