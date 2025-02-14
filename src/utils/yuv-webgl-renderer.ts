export class YUVRenderer {
  constructor(canvas) {
      this.gl = canvas.getContext("webgl");
      this.initGL();
  }

  initGL() {
      const gl = this.gl;
      gl.clearColor(0, 0, 0, 1);
      gl.clear(gl.COLOR_BUFFER_BIT);

      // 创建 WebGL 纹理
      this.yTexture = this.createTexture(gl, 1, 1); // 初始化为1x1大小，稍后更新
      this.uTexture = this.createTexture(gl, 1, 1);
      this.vTexture = this.createTexture(gl, 1, 1);

      // WebGL 着色器
      const vertexShaderSource = `
          attribute vec4 a_position;
          attribute vec2 a_texCoord;
          varying vec2 v_texCoord;
          void main() {
              gl_Position = a_position;
              v_texCoord = a_texCoord;
          }
      `;

      const fragmentShaderSource = `
          precision mediump float;
          varying vec2 v_texCoord;
          uniform sampler2D u_textureY;
          uniform sampler2D u_textureU;
          uniform sampler2D u_textureV;

          void main() {
              float y = texture2D(u_textureY, v_texCoord).r;
              float u = texture2D(u_textureU, v_texCoord).r - 0.5;
              float v = texture2D(u_textureV, v_texCoord).r - 0.5;

              float r = y + 1.402 * v;
              float g = y - 0.344 * u - 0.714 * v;
              float b = y + 1.772 * u;

              gl_FragColor = vec4(r, g, b, 1.0);
          }
      `;

      this.program = this.createShaderProgram(gl, vertexShaderSource, fragmentShaderSource);
      gl.useProgram(this.program);

      // 绑定纹理到着色器
      gl.uniform1i(gl.getUniformLocation(this.program, "u_textureY"), 0);
      gl.uniform1i(gl.getUniformLocation(this.program, "u_textureU"), 1);
      gl.uniform1i(gl.getUniformLocation(this.program, "u_textureV"), 2);
  }

  createTexture(gl, width, height) {
      const texture = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.LUMINANCE, width, height, 0, gl.LUMINANCE, gl.UNSIGNED_BYTE, null);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      return texture;
  }

  createShaderProgram(gl, vertexSource, fragmentSource) {
      function compileShader(gl, source, type) {
          const shader = gl.createShader(type);
          gl.shaderSource(shader, source);
          gl.compileShader(shader);
          return shader;
      }
      const vertexShader = compileShader(gl, vertexSource, gl.VERTEX_SHADER);
      const fragmentShader = compileShader(gl, fragmentSource, gl.FRAGMENT_SHADER);
      const program = gl.createProgram();
      gl.attachShader(program, vertexShader);
      gl.attachShader(program, fragmentShader);
      gl.linkProgram(program);
      return program;
  }

  updateTexture(texture, index, width, height, data) {
      const gl = this.gl;
      gl.activeTexture(gl.TEXTURE0 + index);
      gl.bindTexture(gl.TEXTURE_2D, texture);
      // Ensure the data is valid and not empty
      if (data && data.length > 0) {
          gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, width, height, gl.LUMINANCE, gl.UNSIGNED_BYTE, data);
      } else {
          console.error("Invalid data for texture update.");
      }
  }

  renderFrame(yData, uData, vData, width, height) {
      // Ensure the data is valid
      if (!yData || !uData || !vData || yData.length === 0 || uData.length === 0 || vData.length === 0) {
          console.error("Y, U, or V data is empty!");
          return;
      }

      // Update the textures with Y, U, V data
      this.updateTexture(this.yTexture, 0, width, height, yData);
      this.updateTexture(this.uTexture, 1, width / 2, height / 2, uData);
      this.updateTexture(this.vTexture, 2, width / 2, height / 2, vData);

      // Draw the frame
      this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
  }
}
