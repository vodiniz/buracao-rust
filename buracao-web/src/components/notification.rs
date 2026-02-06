use leptos::prelude::*;

// 1. Criamos um Enum para diferenciar os tipos
#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Info,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType, // 2. Adicionamos o tipo na estrutura
}

#[component]
pub fn NotificationToast(#[prop(into)] toasts: Signal<Vec<Toast>>) -> impl IntoView {
    view! {
        <style>
            "
            @keyframes fadeOutAnimation {
                0% { opacity: 0; transform: translateY(20px); }
                10% { opacity: 1; transform: translateY(0); }
                80% { opacity: 1; }
                100% { opacity: 0; }
            }
            .toast-item {
                animation: fadeOutAnimation 4s forwards;
            }
            "
        </style>

        <div style="
            position: fixed;
            bottom: 220px;
            right: 20px;
            display: flex;
            flex-direction: column;
            align-items: flex-end;
            gap: 10px;
            z-index: 1000;
            pointer-events: none;
        ">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    // 3. Lógica visual baseada no tipo
                    let (bg_color, border_color) = match toast.toast_type {
                        ToastType::Info => ("rgba(0, 0, 0, 0.7)", "#ffeb3b"), // Preto/Amarelo
                        ToastType::Error => ("rgba(183, 28, 28, 0.9)", "#ffffff"), // Vermelho/Branco
                    };

                    view! {
                        <div class="toast-item" style=format!("
                            background-color: {};
                            color: white;
                            padding: 12px 20px;
                            border-radius: 8px;
                            font-size: 14px;
                            border-left: 5px solid {};
                            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
                            backdrop-filter: blur(2px);
                            max-width: 300px;
                            word-wrap: break-word;
                            font-weight: 500;
                        ", bg_color, border_color)>
                            {toast.message}
                        </div>
                    }
                }
            />
        </div>
    }
}
