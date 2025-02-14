export class WebGLRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.gl = null;
    this.texture = null;
    this.initWebGL();
  }

  initWebGL() {
    this.gl = this.canvas.getContext("webgl");
    if (!this.gl) {
      console.error("WebGL 不受支持");
      return;
    }

    // 创建 WebGL 着色器
    const vertexShader = this.createShader(this.gl.VERTEX_SHADER, vertexShaderSource);
    const fragmentShader = this.createShader(this.gl.FRAGMENT_SHADER, fragmentShaderSource);

    // 创建 WebGL 程序
    const program = this.createProgram(vertexShader, fragmentShader);
    this.gl.useProgram(program);

    // 设置顶点坐标和纹理坐标
    const vertices = new Float32Array([
      -1, -1, 0, 0, // 左下角
      1, -1, 1, 0, // 右下角
      -1, 1, 0, 1, // 左上角
      1, 1, 1, 1, // 右上角
    ]);

    const buffer = this.gl.createBuffer();
    this.gl.bindBuffer(this.gl.ARRAY_BUFFER, buffer);
    this.gl.bufferData(this.gl.ARRAY_BUFFER, vertices, this.gl.STATIC_DRAW);

    const positionLocation = this.gl.getAttribLocation(program, "position");
    const texcoordLocation = this.gl.getAttribLocation(program, "texcoord");

    this.gl.enableVertexAttribArray(positionLocation);
    this.gl.vertexAttribPointer(positionLocation, 2, this.gl.FLOAT, false, 16, 0);

    this.gl.enableVertexAttribArray(texcoordLocation);
    this.gl.vertexAttribPointer(texcoordLocation, 2, this.gl.FLOAT, false, 16, 8);

    // 创建 WebGL 纹理
    this.texture = this.gl.createTexture();
    this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.LINEAR);
    this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.LINEAR);

    this.gl.useProgram(program);
  }

  createShader(type, source) {
    const shader = this.gl.createShader(type);
    this.gl.shaderSource(shader, source);
    this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) {
      console.error("An error occurred compiling the shaders: " + this.gl.getShaderInfoLog(shader));
      this.gl.deleteShader(shader);
      return null;
    }
    return shader;
  }

  createProgram(vertexShader, fragmentShader) {
    const program = this.gl.createProgram();
    this.gl.attachShader(program, vertexShader);
    this.gl.attachShader(program, fragmentShader);
    this.gl.linkProgram(program);
    if (!this.gl.getProgramParameter(program, this.gl.LINK_STATUS)) {
      console.error("Unable to initialize the shader program: " + this.gl.getProgramInfoLog(program));
      return null;
    }
    return program;
  }

  updateFrame([width, height, data]) {
    if (!this.gl || !this.canvas) return;

    this.canvas.width = width;
    this.canvas.height = height;

    this.gl.viewport(0, 0, this.canvas.width, this.canvas.height);

    this.gl.bindTexture(this.gl.TEXTURE_2D, this.texture);

    if (!this.gl.texImageInitialized) {
      // **初始化时只调用一次 texImage2D**
      this.gl.texImage2D(
        this.gl.TEXTURE_2D,
        0,
        this.gl.RGB,
        width,
        height,
        0,
        this.gl.RGB,
        this.gl.UNSIGNED_BYTE,
        new Uint8Array(data)
      );
      this.gl.texImageInitialized = true;
    } else {
      // **之后只用 texSubImage2D 更新数据**
      this.gl.texSubImage2D(
        this.gl.TEXTURE_2D,
        0,
        0,
        0,
        width,
        height,
        this.gl.RGB,
        this.gl.UNSIGNED_BYTE,
        new Uint8Array(data)
      );
    }

    this.gl.clear(this.gl.COLOR_BUFFER_BIT);
    this.gl.drawArrays(this.gl.TRIANGLE_STRIP, 0, 4);
  }
}

const vertexShaderSource = `
  attribute vec2 position;
  attribute vec2 texcoord;
  varying vec2 v_texcoord;
  void main() {
      gl_Position = vec4(position, 0.0, 1.0);
      v_texcoord = vec2(texcoord.x, 1.0 - texcoord.y);
  }
`;

const fragmentShaderSource = `
  precision mediump float;
  varying vec2 v_texcoord;
  uniform sampler2D texture;
  void main() {
    gl_FragColor = texture2D(texture, v_texcoord);
  }
`;
