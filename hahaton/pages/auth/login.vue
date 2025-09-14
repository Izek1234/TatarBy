<template>
  <div class="auth-container">
    <div class="auth-card">
      <div class="auth-header">
        <h1>{{ isLogin ? 'Вход в систему' : 'Регистрация' }}</h1>
        <p>{{ isLogin ? 'Введите свои данные для входа' : 'Создайте новую учетную запись' }}</p>
      </div>

      <form @submit.prevent="submitForm" class="auth-form">
        <div class="form-group">
          <label for="email">Email</label>
          <input
            type="email"
            id="email"
            v-model="form.email"
            required
            placeholder="Введите ваш email"
            :class="{ 'error': errors.email }"
          >
          <span class="error-message" v-if="errors.email">{{ errors.email }}</span>
        </div>

        <div class="form-group" v-if="!isLogin">
          <label for="name">Имя</label>
          <input
            type="text"
            id="name"
            v-model="form.name"
            required
            placeholder="Введите ваше имя"
            :class="{ 'error': errors.name }"
          >
          <span class="error-message" v-if="errors.name">{{ errors.name }}</span>
        </div>

        <div class="form-group">
          <label for="password">Пароль</label>
          <input
            :type="showPassword ? 'text' : 'password'"
            id="password"
            v-model="form.password"
            required
            :placeholder="isLogin ? 'Введите ваш пароль' : 'Создайте надежный пароль'"
            :class="{ 'error': errors.password }"
          >
          <button type="button" class="password-toggle" @click="showPassword = !showPassword">
            <span v-if="showPassword">👁️</span>
            <span v-else>👁️</span>
          </button>
          <span class="error-message" v-if="errors.password">{{ errors.password }}</span>
        </div>

        <div class="form-group" v-if="!isLogin">
          <label for="confirmPassword">Подтверждение пароля</label>
          <input
            :type="showConfirmPassword ? 'text' : 'password'"
            id="confirmPassword"
            v-model="form.confirmPassword"
            required
            placeholder="Повторите ваш пароль"
            :class="{ 'error': errors.confirmPassword }"
          >
          <button type="button" class="password-toggle" @click="showConfirmPassword = !showConfirmPassword">
            <span v-if="showConfirmPassword">👁️</span>
            <span v-else>👁️</span>
          </button>
          <span class="error-message" v-if="errors.confirmPassword">{{ errors.confirmPassword }}</span>
        </div>

        <button type="submit" class="submit-btn" :disabled="loading">
          <span v-if="loading">
            <div class="spinner"></div>
            Загрузка...
          </span>
          <span v-else>{{ isLogin ? 'Войти' : 'Зарегистрироваться' }}</span>
        </button>

        <div class="auth-footer">
          <p>
            {{ isLogin ? 'Еще нет аккаунта?' : 'Уже есть аккаунт?' }}
            <a href="#" @click.prevent="toggleMode">
              {{ isLogin ? 'Зарегистрироваться' : 'Войти' }}
            </a>
          </p>
        </div>
      </form>

      <div class="auth-alert" :class="alert.type" v-if="alert.message">
        {{ alert.message }}
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'AuthPage',
  data() {
    return {
      isLogin: true,
      loading: false,
      showPassword: false,
      showConfirmPassword: false,
      form: {
        email: '',
        name: '',
        password: '',
        confirmPassword: ''
      },
      errors: {},
      alert: {
        type: '',
        message: ''
      }
    }
  },
  methods: {
    toggleMode() {
      this.isLogin = !this.isLogin
      this.resetForm()
      this.clearErrors()
      this.hideAlert()
    },
    
    async submitForm() {
      if (!this.validateForm()) return
      
      this.loading = true
      this.clearErrors()
      this.hideAlert()

      try {
        const url = this.isLogin ? '/api/auth/login' : '/api/auth/register'
        const response = await this.$fetchApi(url, {
          method: 'POST',
          body: JSON.stringify(this.prepareFormData())
        })

        if (response.success) {
          await this.handleSuccessResponse(response)
        } else {
          this.handleApiError(response)
        }
      } catch (error) {
        this.handleNetworkError(error)
      } finally {
        this.loading = false
      }
    },

    prepareFormData() {
      if (this.isLogin) {
        return {
          email: this.form.email,
          password: this.form.password
        }
      }
      return {
        email: this.form.email,
        name: this.form.name,
        password: this.form.password
      }
    },

    async handleSuccessResponse(response) {
      // Сохраняем токен и данные пользователя
      this.$store.commit('auth/setUser', response.data.user)
      this.$store.commit('auth/setToken', response.data.token)
      
      // Сохраняем токен в localStorage
      if (process.client) {
        localStorage.setItem('auth_token', response.data.token)
        localStorage.setItem('user_data', JSON.stringify(response.data.user))
      }
      
      this.showAlert('Успешная авторизация!', 'success')
      await this.$router.push('/dashboard')
    },

    handleApiError(response) {
      if (response.errors) {
        this.errors = response.errors
      } else if (response.message) {
        this.showAlert(response.message, 'error')
      }
    },

    handleNetworkError(error) {
      console.error('Network error:', error)
      this.showAlert('Ошибка сети. Проверьте подключение к интернету.', 'error')
    },

    validateForm() {
      this.errors = {}
      let isValid = true

      // Валидация email
      if (!this.form.email) {
        this.errors.email = 'Email обязателен'
        isValid = false
      } else if (!/\S+@\S+\.\S+/.test(this.form.email)) {
        this.errors.email = 'Некорректный формат email'
        isValid = false
      }

      // Валидация имени (только для регистрации)
      if (!this.isLogin && !this.form.name) {
        this.errors.name = 'Имя обязательно'
        isValid = false
      } else if (!this.isLogin && this.form.name.length < 2) {
        this.errors.name = 'Имя должно содержать минимум 2 символа'
        isValid = false
      }

      // Валидация пароля
      if (!this.form.password) {
        this.errors.password = 'Пароль обязателен'
        isValid = false
      } else if (this.form.password.length < 6) {
        this.errors.password = 'Пароль должен содержать минимум 6 символов'
        isValid = false
      }

      // Подтверждение пароля (только для регистрации)
      if (!this.isLogin && this.form.password !== this.form.confirmPassword) {
        this.errors.confirmPassword = 'Пароли не совпадают'
        isValid = false
      }

      return isValid
    },

    showAlert(message, type = 'error') {
      this.alert = { message, type }
      setTimeout(() => this.hideAlert(), 5000)
    },

    hideAlert() {
      this.alert = { message: '', type: '' }
    },

    clearErrors() {
      this.errors = {}
    },

    resetForm() {
      this.form = {
        email: '',
        name: '',
        password: '',
        confirmPassword: ''
      }
    }
  }
}
</script>

<style scoped>
.auth-container {
  display: flex;
  height: 100vh;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #268d34 0%, #4ba26c 100%);
  padding: 20px;
}

.auth-card {
  background: white;
  border-radius: 12px;
  box-shadow: 0 15px 35px rgba(0, 0, 0, 0.1);
  padding: 40px;
  width: 100%;
  max-width: 400px;
  backdrop-filter: blur(10px);
}

.auth-header {
  text-align: center;
  margin-bottom: 30px;
}

.auth-header h1 {
  color: #333;
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 8px;
}

.auth-header p {
  color: #666;
  font-size: 14px;
}

.form-group {
  margin-bottom: 20px;
  position: relative;
}

label {
  display: block;
  margin-bottom: 8px;
  color: #333;
  font-weight: 500;
  font-size: 14px;
}

input {
  width: 100%;
  padding: 12px 16px;
  border: 2px solid #e1e5e9;
  border-radius: 8px;
  font-size: 14px;
  transition: all 0.3s ease;
  box-sizing: border-box;
}

input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

input.error {
  border-color: #e74c3c;
}

.password-toggle {
  position: absolute;
  right: 12px;
  top: 38px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 16px;
  padding: 0;
}

.error-message {
  color: #e74c3c;
  font-size: 12px;
  margin-top: 4px;
  display: block;
}

.submit-btn {
  width: 100%;
  padding: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.submit-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 5px 15px rgba(102, 126, 234, 0.4);
}

.submit-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid transparent;
  border-top: 2px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.auth-footer {
  text-align: center;
}

.auth-footer p {
  color: #666;
  font-size: 14px;
}

.auth-footer a {
  color: #667eea;
  text-decoration: none;
  font-weight: 500;
}

.auth-footer a:hover {
  text-decoration: underline;
}

.auth-alert {
  padding: 12px;
  border-radius: 8px;
  margin-top: 20px;
  font-size: 14px;
  text-align: center;
}

.auth-alert.error {
  background-color: #fee;
  color: #e74c3c;
  border: 1px solid #f5c6cb;
}

.auth-alert.success {
  background-color: #eff;
  color: #27ae60;
  border: 1px solid #c3e6cb;
}

@media (max-width: 480px) {
  .auth-card {
    padding: 30px 20px;
  }
  
  .auth-header h1 {
    font-size: 20px;
  }
}
</style>