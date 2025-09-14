<template>
  <div class="chat-container">
    <!-- Заголовок с информацией о пользователе -->
    <div class="chat-header">
      <div class="header-content">
        <div class="user-info">
          <div class="user-avatar">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
              <circle cx="12" cy="7" r="4"></circle>
            </svg>
          </div>
          <div class="user-details">
            <span class="user-name">Габдуллин Шамиль</span>
            <span class="user-status">Онлайн</span>
          </div>
        </div>
        
        <div class="logo">
          <div class="logo-icon">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2L2 7l10 5 10-5-10-5z"></path>
              <path d="M2 17l10 5 10-5"></path>
              <path d="M2 12l10 5 10-5"></path>
            </svg>
          </div>
          <div class="logo-text">
            <h1>ТатGPT</h1>
            <p>Татар AI Ассистенты</p>
          </div>
        </div>
        
        <button class="clear-btn" @click="clearChat" title="Чатны чистарту">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"></path>
          </svg>
        </button>
      </div>
    </div>

    <!-- Татарский орнамент -->
    <div class="tatar-pattern top-pattern"></div>

    <!-- Сообщения -->
    <div class="messages-container" ref="messagesContainer">
      <!-- Приветственное сообщение -->
      <div class="welcome-message" v-if="messages.length === 0">
        <div class="welcome-content">
          <div class="welcome-icon">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2L2 7l10 5 10-5-10-5z"></path>
              <path d="M2 17l10 5 10-5"></path>
              <path d="M2 12l10 5 10-5"></path>
            </svg>
          </div>
          <h3>Рәхим итегез!</h3>
          <p>Татар телендә AI ярдәмче белән сөйләшүне башлап җибәрегез</p>
          <div class="suggestions">
            <button @click="sendSuggestion('Татарстан турында сөйләшегез')">Татарстан турында</button>
            <button @click="sendSuggestion('Татар телен өйрәнү')">Татар телен өйрәнү</button>
            <button @click="sendSuggestion('Татар йолалары турында')">Татар йолалары</button>
          </div>
        </div>
      </div>

      <!-- Сообщения чата -->
      <div
        v-for="message in messages"
        :key="message.id"
        :class="['message', message.type]"
      >
        <div class="message-content">
          <div :class="['message-bubble', message.type]">
            <p>{{ message.content }}</p>
          </div>
          <div class="message-time">{{ message.timestamp }}</div>
        </div>
      </div>

      <!-- Индикатор набора -->
      <div class="typing-indicator" v-if="isTyping">
        <div class="typing-content">
          <div class="typing-bubble ai">
            <div class="typing-dots">
              <div class="dot"></div>
              <div class="dot" style="animation-delay: 0.1s"></div>
              <div class="dot" style="animation-delay: 0.2s"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Генерированное изображение -->
      <div class="generated-image-container" v-if="showImageGenerator && generatedImage">
        <div class="generated-image-wrapper">
          <img src="../../public/sun_in_sky.png" :alt="'Сгенерированная картинка'" class="generated-image" />
          <div class="image-caption">Сгенерировано по запросу</div>
        </div>
      </div>

      <div ref="messagesEndRef"></div>
    </div>

    <!-- Татарский орнамент -->
    <div class="tatar-pattern bottom-pattern"></div>

    <!-- Поле ввода -->
    <div class="input-container">
      <div class="input-wrapper">
        <div class="input-actions">
          <button class="action-btn" title="Татар телендә ярдәм">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"></path>
            </svg>
          </button>
        </div>
        <textarea
          v-model="inputValue"
          @keypress="handleKeyPress"
          @input="adjustTextareaHeight"
          placeholder="Язма кертегез..."
          ref="textarea"
          class="input-field"
          rows="1"
          :disabled="isTyping"
        ></textarea>
        <button
          class="send-btn"
          @click="handleSendMessage"
          :disabled="!inputValue.trim() || isTyping"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
          </svg>
        </button>
      </div>

      <!-- Кнопка генерации изображения -->
      <div class="image-gen-button-container">
        <button
          class="image-gen-btn"
          @click="generateImage"
          :disabled="!inputValue.trim() || isTyping"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <circle cx="8.5" cy="8.5" r="1.5"></circle>
            <polyline points="21 15 16 10 5 21"></polyline>
          </svg>
          <span>Сгенерировать картинку</span>
        </button>
      </div>

     
    </div>
  </div>
</template>

<script>
export default {
  name: 'TatarChat',
  data() {
    return {
      messages: [
        {
          id: 1,
          type: 'ai',
          content: 'Сәлам! Мин сезнең Татар AI-ярдәмчегыз. Бүген сезгә ничек ярдәм итә алам?',
          timestamp: this.getCurrentTime()
        }
      ],
      inputValue: '',
      isTyping: false,
      showImageGenerator: false,
      generatedImage: null
    }
  },
  mounted() {
    this.scrollToBottom()
    this.adjustTextareaHeight()
  },
  methods: {
    scrollToBottom() {
      this.$nextTick(() => {
        const container = this.$refs.messagesEndRef
        if (container) {
          container.scrollIntoView({ behavior: "smooth" })
        }
      })
    },

    adjustTextareaHeight() {
      this.$nextTick(() => {
        const textarea = this.$refs.textarea
        if (textarea) {
          textarea.style.height = 'auto'
          textarea.style.height = Math.min(textarea.scrollHeight, 120) + 'px'
        }
      })
    },

    async handleSendMessage() {
      if (!this.inputValue.trim() || this.isTyping) return

      const userMessage = {
        id: Date.now(),
        type: 'user',
        content: this.inputValue.trim(),
        timestamp: this.getCurrentTime()
      }

      this.messages.push(userMessage)
      this.inputValue = ''
      this.adjustTextareaHeight()
      this.isTyping = true

      // Имитация ответа AI
      setTimeout(() => {
         const aiResponses = [
          "Су Анасы (Су Иясе)\n\nБу халык әкияте бер ялгыз иптәшлектә яшәгән ана белән угыл турында. Углы, яшь егет, бер көнне елгада балык тотарга бара. Ул бик зур һәм матур балык тота, ләкин балык сөйли башлый һәм үзенең Су Анасының кызы икәнен әйтеп, аны куйгыласын ялый. Карышкыр йөрәкле егет аны ишетми һәм аны үз өенә алып кита.\n\nӨйгә кайткач, анасы балыкны пешерергә әзерли. Ләкин балык пешеренү вакытында яңадан сөйли башлый һәм аны ашамаска яланый. Анасы куркып, балыкны пешерүдән баш тарта, әмма егет тагын да катырак пешерергә куша.\n\nНиһаять, балык пешеп беткәч, аны ашап бетерәләр. Шул төндә үк, куркыныч Су Анасы аларның өенә килеп чыга. Ул бик ачулы һәм үзенең кызы өчен үч алырга ант итә. Ул ана-угылны суга тартып алырга яный, әмма алар эченә керә алмый.\n\nАхырында, Су Анасы ачуыннан үзе белән килгән суны өйгә сибеп, бастырып куя. Икәве дә судан үлеп бетә. Шул көннән бирле, әлеге өйдә беркем дә яши алмый, ул яшәүдән төшеп, елан-чарлар белән тулып китә.\n\nТөп тема: Ачкалык, байлык өчен башкаларның теләгенә каршы бару, табигатькә хөрмәт итү мәҗбүрилеге һәм ґәзаб китерә торган явыз эшләрнең ахыргы нәтиҗәсе."
        ]

        const randomResponse = aiResponses[Math.floor(Math.random() * aiResponses.length)]

        const aiMessage = {
          id: Date.now() + 1,
          type: 'ai',
          content: randomResponse,
          timestamp: this.getCurrentTime()
        }

        this.messages.push(aiMessage)
        this.isTyping = false
      }, 1500)
    },

    generateImage() {
      if (!this.inputValue.trim()) return

      const prompt = this.inputValue || "татар орнаменты белән абстракт сәнгать"
      const imageUrl = `https://placehold.co/600x400/0a3d2b/ffffff?text=Генерированная+картинка:+${encodeURIComponent(prompt)}`
      
      this.generatedImage = imageUrl
      this.showImageGenerator = true

      const aiMessage = {
        id: Date.now() + 1,
        type: 'ai',
        content: `Мин сезнең соравыгыз буенча рәсем төзедем: "${prompt}"`,
        timestamp: this.getCurrentTime()
      }

      this.messages.push(aiMessage)
      this.inputValue = '' // Очищаем поле после генерации
      this.adjustTextareaHeight()
    },

    getCurrentTime() {
      const now = new Date()
      return now.getHours().toString().padStart(2, '0') + ':' + now.getMinutes().toString().padStart(2, '0')
    },

    handleKeyPress(e) {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        this.handleSendMessage()
      }
    },

    clearChat() {
      this.messages = [
        {
          id: 1,
          type: 'ai',
          content: 'Сәлам! Мин сезнең Татар AI-ярдәмчегыз. Бүген сезгә ничек ярдәм итә алам?',
          timestamp: this.getCurrentTime()
        }
      ]
      this.generatedImage = null
      this.showImageGenerator = false
    },

    sendSuggestion(text) {
      this.inputValue = text
      this.handleSendMessage()
    }
  },

  watch: {
    messages: {
      handler() {
        this.scrollToBottom()
      },
      deep: true
    }
  }
}
</script>

<style scoped>
.chat-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #0a1a15 0%, #152a20 50%, #1a3025 100%);
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  color: #fff;
  position: relative;
  overflow: hidden;
}

/* Татарский орнамент */
.tatar-pattern {
  position: absolute;
  left: 0;
  right: 0;
  height: 20px;
  background-image: 
    radial-gradient(circle, rgba(76, 145, 92, 0.3) 3px, transparent 3px),
    radial-gradient(circle, rgba(76, 145, 92, 0.2) 2px, transparent 2px);
  background-size: 40px 40px;
  background-position: 0 0, 20px 20px;
  z-index: 1;
}

.top-pattern {
  top: 80px;
}

.bottom-pattern {
  bottom: 140px;
}

/* Заголовок */
.chat-header {
  background: rgba(10, 26, 21, 0.8);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(76, 145, 92, 0.3);
  padding: 16px 24px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 10;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 800px;
  margin: 0 auto;
  position: relative;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 10px;
  position: absolute;
  left: 0;
}

.user-avatar {
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #8d5dff 0%, #d05ad1 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.user-avatar svg {
  width: 18px;
  height: 18px;
}

.user-details {
  display: flex;
  flex-direction: column;
}

.user-name {
  font-size: 14px;
  font-weight: 600;
  color: white;
}

.user-status {
  font-size: 11px;
  color: #4c915c;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 0 auto;
}

.logo-icon {
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #4c915c 0%, #2d6b3d 100%);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.logo-icon svg {
  width: 24px;
  height: 24px;
}

.logo-text h1 {
  color: #84cc7f;
  font-size: 20px;
  font-weight: 700;
  margin: 0;
  letter-spacing: 0.5px;
}

.logo-text p {
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  margin: 0;
  font-weight: 400;
}

.clear-btn {
  background: rgba(76, 145, 92, 0.2);
  border: 1px solid rgba(76, 145, 92, 0.3);
  border-radius: 8px;
  padding: 8px;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  position: absolute;
  right: 0;
}

.clear-btn:hover {
  background: rgba(76, 145, 92, 0.3);
  transform: scale(1.05);
}

.clear-btn svg {
  width: 20px;
  height: 20px;
}

/* Контейнер сообщений */
.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
  scroll-behavior: smooth;
  position: relative;
  z-index: 2;
}

/* Приветственное сообщение */
.welcome-message {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  opacity: 0.9;
}

.welcome-content {
  text-align: center;
  color: rgba(255, 255, 255, 0.9);
  max-width: 500px;
}

.welcome-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  color: #4c915c;
}

.welcome-icon svg {
  width: 100%;
  height: 100%;
}

.welcome-content h3 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 12px 0;
  color: white;
}

.welcome-content p {
  font-size: 16px;
  margin: 0 0 24px 0;
  opacity: 0.8;
  line-height: 1.5;
}

.suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: center;
  margin-top: 20px;
}

.suggestions button {
  background: rgba(76, 145, 92, 0.15);
  border: 1px solid rgba(76, 145, 92, 0.3);
  border-radius: 18px;
  padding: 10px 16px;
  color: white;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
  border: none;
}

.suggestions button:hover {
  background: rgba(76, 145, 92, 0.3);
  transform: translateY(-2px);
}

/* Сообщения */
.message {
  display: flex;
  margin-bottom: 20px;
}

.message.user {
  justify-content: flex-end;
}

.message-content {
  max-width: 70%;
  display: flex;
  flex-direction: column;
}

.message.user .message-content {
  align-items: flex-end;
}

.message.ai .message-content {
  align-items: flex-start;
}

.message-bubble {
  padding: 16px 20px;
  border-radius: 20px;
  line-height: 1.4;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.message-bubble.user {
  background: linear-gradient(135deg, #4c915c 0%, #2d6b3d 100%);
  color: white;
  border-bottom-right-radius: 6px;
}

.message-bubble.ai {
  background: rgba(255, 255, 255, 0.05);
  color: #a8e6cf;
  border-bottom-left-radius: 6px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(76, 145, 92, 0.2);
}

.message-bubble p {
  margin: 0;
  font-size: 15px;
  line-height: 1.5;
  font-weight: 500;
}

.message-time {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  margin-top: 6px;
  padding: 0 8px;
}

.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
  scroll-behavior: smooth;
  position: relative;
  
  /* Скрываем полосу прокрутки */
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE и Edge */
}

.messages-container::-webkit-scrollbar {
  display: none; /* Chrome, Safari, Opera */
}

/* Индикатор набора */
.typing-indicator {
  display: flex;
  margin-bottom: 16px;
  justify-content: start;
}

.typing-content {
  max-width: 70%;
}

.typing-bubble.ai {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  border-bottom-left-radius: 6px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(76, 145, 92, 0.2);
  padding: 16px 20px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.typing-dots {
  display: flex;
  gap: 4px;
}

.dot {
  width: 6px;
  height: 6px;
  background: #4c915c;
  border-radius: 50%;
  animation: typing 1.4s infinite ease-in-out;
}

@keyframes typing {
  0%, 80%, 100% {
    transform: scale(0.8);
    opacity: 0.5;
  } 
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Генерированное изображение */
.generated-image-container {
  display: flex;
  justify-content: center;
  margin: 20px 0;
}

.generated-image-wrapper {
  border-radius: 20px;
  overflow: hidden;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(76, 145, 92, 0.3);
}

.generated-image {
  max-width: 600px;
  height: auto;
  object-fit: contain;
  display: block;
}

.image-caption {
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.05);
  color: #a8e6cf;
  font-size: 12px;
  text-align: center;
  font-weight: 500;
}

/* Поле ввода */
.input-container {
  background: rgba(10, 26, 21, 0.8);
  backdrop-filter: blur(20px);
  border-top: 1px solid rgba(76, 145, 92, 0.3);
  padding: 20px 24px 24px;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.1);
  z-index: 10;
  margin-top: auto;
}

.input-wrapper {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  max-width: 800px;
  margin: 0 auto;
}

.input-actions {
  display: flex;
}

.action-btn {
  background: transparent;
  border: none;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  color: white;
  background: rgba(76, 145, 92, 0.2);
}

.action-btn svg {
  width: 20px;
  height: 20px;
}

.input-field {
  flex: 1;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(76, 145, 92, 0.3);
  border-radius: 20px;
  padding: 16px;
  font-size: 15px;
  font-family: inherit;
  resize: none;
  outline: none;
  color: #a8e6cf;
  max-height: 120px;
  min-height: 56px;
  transition: all 0.2s;
  caret-color: #4c915c;
}

.input-field:focus {
  border-color: rgba(76, 145, 92, 0.6);
  background: rgba(255, 255, 255, 0.1);
}

.input-field::placeholder {
  color: rgba(255, 255, 255, 0.5);
}

.send-btn {
  background: linear-gradient(135deg, #4c915c 0%, #2d6b3d 100%);
  border: none;
  border-radius: 50%;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.send-btn:hover:not(:disabled) {
  transform: scale(1.05);
  box-shadow: 0 6px 12px rgba(45, 107, 61, 0.4);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.send-btn svg {
  width: 20px;
  height: 20px;
}

/* Кнопка генерации изображения */
.image-gen-button-container {
  display: flex;
  justify-content: center;
  margin-top: 12px;
}

.image-gen-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: linear-gradient(135deg, #8d5dff 0%, #d05ad1 100%);
  border: none;
  border-radius: 20px;
  padding: 12px 24px;
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.image-gen-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(141, 93, 255, 0.4);
}

.image-gen-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.image-gen-btn svg {
  width: 20px;
  height: 20px;
}

.language-note {
  text-align: center;
  margin-top: 16px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
}

/* Адаптивность */
@media (max-width: 768px) {
  .chat-header {
    padding: 12px 16px;
  }
  
  .user-info {
    position: static;
    margin-right: auto;
  }
  
  .header-content {
    flex-wrap: wrap;
    gap: 10px;
  }
  
  .logo {
    order: -1;
    width: 100%;
    justify-content: center;
    margin-bottom: 10px;
  }

  .messages-container {
    padding: 16px;
  }

  .input-container {
    padding: 16px;
  }

  .message-content {
    max-width: 85%;
  }

  .logo-text h1 {
    font-size: 18px;
  }

  .suggestions {
    flex-direction: column;
    align-items: center;
  }

  .suggestions button {
    width: 100%;
    max-width: 280px;
  }

  .input-wrapper {
    flex-direction: column;
  }

  .image-gen-button-container {
    margin-top: 10px;
  }

  .generated-image-wrapper {
    max-width: 90%;
  }
  
  .tatar-pattern {
    display: none;
  }
}
</style>