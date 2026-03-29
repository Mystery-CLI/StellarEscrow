export class XSSProtection {
  static preventXSS(html: string): string {
    const div = document.createElement('div');
    div.textContent = html;
    return div.innerHTML;
  }

  static sanitizeAttribute(attr: string): string {
    return attr
      .replace(/javascript:/gi, '')
      .replace(/on\w+\s*=/gi, '')
      .replace(/[<>]/g, '');
  }

  static validateScriptTag(tag: string): boolean {
    return !/<script[^>]*>[\s\S]*?<\/script>/gi.test(tag);
  }

  static removeScriptTags(html: string): string {
    return html.replace(/<script[^>]*>[\s\S]*?<\/script>/gi, '');
  }

  static preventClickjacking(): void {
    if (window.self !== window.top) {
      window.top!.location = window.self.location;
    }
  }

  static preventFrameInjection(): void {
    const meta = document.createElement('meta');
    meta.httpEquiv = 'X-Frame-Options';
    meta.content = 'DENY';
    document.head.appendChild(meta);
  }
}

export class CSRFProtection {
  private static token: string | null = null;

  static generateToken(): string {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    this.token = Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join('');
    return this.token;
  }

  static getToken(): string {
    if (!this.token) {
      this.generateToken();
    }
    return this.token!;
  }

  static validateToken(token: string): boolean {
    return token === this.token;
  }

  static setTokenHeader(headers: Record<string, string>): Record<string, string> {
    headers['X-CSRF-Token'] = this.getToken();
    return headers;
  }
}
